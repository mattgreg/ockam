use crate::relay::{run_mailbox, RelayMessage, ShutdownHandle, ShutdownListener};
use crate::Context;
use ockam_core::{Processor, Result};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

pub struct ProcessorRelay<P>
where
    P: Processor<Context = Context>,
{
    processor: P,
    ctx: Context,
    shutdown_listener: ShutdownListener,
}

impl<P> ProcessorRelay<P>
where
    P: Processor<Context = Context>,
{
    pub fn new(processor: P, ctx: Context, shutdown_listener: ShutdownListener) -> Self {
        Self {
            processor,
            ctx,
            shutdown_listener,
        }
    }

    async fn run(self) {
        let (rx_shutdown, tx_ack) = self.shutdown_listener.consume();
        let mut ctx = self.ctx;
        let mut processor = self.processor;

        match processor.initialize(&mut ctx).await {
            Ok(()) => {}
            Err(e) => {
                error!(
                    "Failure during '{}' processor initialisation: {}",
                    ctx.address(),
                    e
                );
            }
        }

        let run_loop = async {
            loop {
                let should_continue = processor.process(&mut ctx).await?;
                if !should_continue {
                    break;
                }
            }

            Result::<()>::Ok(())
        };

        enum StopReason {
            Shutdown,
            LoopStop,
        }

        let stop_reason;

        tokio::select! {
            Ok(_) = rx_shutdown => { stop_reason = StopReason::Shutdown; }
            Ok(_) = run_loop => { stop_reason = StopReason::LoopStop; }
            else => {
                panic!()
            }
        }

        match processor.shutdown(&mut ctx).await {
            Ok(()) => {}
            Err(e) => {
                error!(
                    "Failure during '{}' processor shutdown: {}",
                    ctx.address(),
                    e
                );
            }
        }

        tx_ack.send(()).unwrap();

        match stop_reason {
            StopReason::Shutdown => {}
            StopReason::LoopStop => { ctx.stop_processor(ctx.address()).await.unwrap(); }
        };
    }

    pub(crate) fn build(
        rt: &Runtime,
        processor: P,
        ctx: Context,
        mb_tx: mpsc::Sender<RelayMessage>,
    ) -> (mpsc::Sender<RelayMessage>, ShutdownHandle)
    where
        P: Processor<Context = Context>,
    {
        let (tx, rx) = mpsc::channel(32);

        let (handle, listener) = ShutdownHandle::create();

        let relay = ProcessorRelay::<P>::new(processor, ctx, listener);

        rt.spawn(run_mailbox(rx, mb_tx));
        rt.spawn(relay.run());

        (tx, handle)
    }
}
