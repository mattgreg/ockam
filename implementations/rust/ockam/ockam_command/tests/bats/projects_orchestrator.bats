#!/bin/bash

# ===== SETUP

setup_file() {
  load load/base.bash
}

setup() {
  load load/base.bash
  load load/orchestrator.bash
  load_bats_ext
  setup_home_dir
  skip_if_orchestrator_tests_not_enabled
  copy_local_orchestrator_data
}

teardown() {
  teardown_home_dir
}

# ===== TESTS
@test "projects - version" {
  run_success "$OCKAM" project version
}

@test "project - enrollment from file - parse check" {
  run_success bash -c "$OCKAM project ticket >$OCKAM_HOME/p.ticket"

  # From file
  run_success "$OCKAM" project enroll "$OCKAM_HOME/p.ticket" --test-argument-parser
  run_failure "$OCKAM" project enroll "$OCKAM_HOME/p.t" --test-argument-parser

  # From contents
  run_success "$OCKAM" project enroll $(cat "$OCKAM_HOME/p.ticket") --test-argument-parser
  run_failure "$OCKAM" project enroll "INVALID_TICKET" --test-argument-parser
}

@test "projects - enrollment" {
  ENROLLED_OCKAM_HOME=$OCKAM_HOME

  setup_home_dir
  NON_ENROLLED_OCKAM_HOME=$OCKAM_HOME

  run_success "$OCKAM" identity create green
  green_identifier=$($OCKAM identity show green)

  run_success "$OCKAM" identity create blue
  blue_identifier=$($OCKAM identity show blue)

  # They haven't been added by enroller yet
  run_failure "$OCKAM" project enroll --identity green --project-path "$PROJECT_JSON_PATH"

  OCKAM_HOME=$ENROLLED_OCKAM_HOME
  $OCKAM project ticket --member "$green_identifier" --attribute role=member
  blue_token=$($OCKAM project ticket --attribute role=member)
  OCKAM_HOME=$NON_ENROLLED_OCKAM_HOME

  # Green' identity was added by enroller
  run_success "$OCKAM" project enroll --identity green --project-path "$PROJECT_JSON_PATH"
  assert_output --partial "$green_identifier"

  # For blue, we use an enrollment token generated by enroller
  run_success "$OCKAM" project enroll $blue_token --identity blue
  assert_output --partial "$blue_identifier"
  OCKAM_HOME=$ENROLLED_OCKAM_HOME
}

@test "projects - access requiring credential" {
  ENROLLED_OCKAM_HOME=$OCKAM_HOME

  # Change to a new home directory where there are no enrolled identities
  setup_home_dir
  NON_ENROLLED_OCKAM_HOME=$OCKAM_HOME

  # Create a named default identity
  run_success "$OCKAM" identity create green
  green_identifier=$($OCKAM identity show green)

  # Create node for the non-enrolled identity using the exported project information
  run_success "$OCKAM" node create green --project-path "$ENROLLED_OCKAM_HOME/project.json"

  # Node can't create relay as it isn't a member
  fwd=$(random_str)
  run_failure "$OCKAM" relay create "$fwd"

  # Add node as a member
  OCKAM_HOME=$ENROLLED_OCKAM_HOME
  run_success "$OCKAM" project ticket --member "$green_identifier" --attribute role=member

  # The node can now access the project's services
  OCKAM_HOME=$NON_ENROLLED_OCKAM_HOME
  fwd=$(random_str)
  run_success "$OCKAM" relay create "$fwd"
}

@test "projects - send a message to a project node from an embedded node, enrolled member on different install" {
  skip # FIXME  how to send a message to a project m1 is enrolled to?  (with m1 being on a different install
  #       than the admin?.  If we pass project' address directly (instead of /project/ thing), would
  #       it present credential? would read authority info from project.json?

  run_success "$OCKAM" project information --output json >/tmp/project.json

  export OCKAM_HOME=/tmp/ockam
  run_success "$OCKAM" identity create m2
  run_success "$OCKAM" identity create m1
  m1_identifier=$($OCKAM identity show m1)

  unset OCKAM_HOME
  run_success "$OCKAM" project ticket --member $m1_identifier --attribute role=member

  export OCKAM_HOME=/tmp/ockam
  # m1' identity was added by enroller
  run_success $OCKAM project enroll --identity m1 --project-path "$PROJECT_JSON_PATH"

  # m1 is a member,  must be able to contact the project' service
  run_success $OCKAM message send --timeout 5 --identity m1 --project-path "$PROJECT_JSON_PATH" --to /project/default/service/echo hello
  assert_output "hello"

  # m2 is not a member,  must not be able to contact the project' service
  run_failure $OCKAM message send --timeout 5 --identity m2 --project-path "$PROJECT_JSON_PATH" --to /project/default/service/echo hello
}

@test "projects - list addons" {
  run_success "$OCKAM" project addon list --project default
  assert_output --partial "Id: okta"
}

@test "projects - enable and disable addons" {
  skip # TODO: wait until cloud has the influxdb and confluent addons enabled

  run_success "$OCKAM" project addon list --project default
  assert_output --partial --regex "Id: okta\n +Enabled: false"
  assert_output --partial --regex "Id: confluent\n +Enabled: false"

  run_success "$OCKAM" project addon enable okta --project default --tenant tenant --client-id client_id --cert cert
  run_success "$OCKAM" project addon enable confluent --project default --bootstrap-server bootstrap-server.confluent:9092 --api-key ApIkEy --api-secret ApIsEcrEt

  run_success "$OCKAM" project addon list --project default
  assert_output --partial --regex "Id: okta\n +Enabled: true"
  assert_output --partial --regex "Id: confluent\n +Enabled: true"

  run_success "$OCKAM" project addon disable --addon okta --project default
  run_success "$OCKAM" project addon disable --addon --project default
  run_success "$OCKAM" project addon disable --addon confluent --project default

  run_success "$OCKAM" project addon list --project default
  assert_output --partial --regex "Id: okta\n +Enabled: false"
  assert_output --partial --regex "Id: confluent\n +Enabled: false"
}

@test "influxdb lease manager" {
  # TODO add more tests
  #      responsible, and that a member enrolled on a different ockam install can access it.
  skip_if_influxdb_test_not_enabled

  run_success "$OCKAM" project addon configure influxdb --org-id "${INFLUXDB_ORG_ID}" --token "${INFLUXDB_TOKEN}" --endpoint-url "${INFLUXDB_ENDPOINT}" --max-ttl 60 --permissions "${INFLUXDB_PERMISSIONS}"

  sleep 30 #FIXME  workaround, project not yet ready after configuring addon

  run_success "$OCKAM" project information default --output json >/tmp/project.json

  export OCKAM_HOME=/tmp/ockam
  run_success "$OCKAM" identity create m1
  run_success "$OCKAM" identity create m2
  run_success "$OCKAM" identity create m3

  m1_identifier=$($OCKAM identity show m1)
  m2_identifier=$($OCKAM identity show m2)

  unset OCKAM_HOME
  run_success "$OCKAM" project ticket --member $m1_identifier --attribute service=sensor
  run_success "$OCKAM" project ticket --member $m2_identifier --attribute service=web

  export OCKAM_HOME=/tmp/ockam

  # m1 and m2 identity was added by enroller
  run_success "$OCKAM" project enroll --identity m1 --project-path "$PROJECT_JSON_PATH"
  assert_output --partial $green_identifier

  run_success "$OCKAM" project enroll --identity m2 --project-path "$PROJECT_JSON_PATH"
  assert_output --partial $green_identifier

  # m1 and m2 can use the lease manager
  run_success "$OCKAM" lease --identity m1 --project-path "$PROJECT_JSON_PATH" create
  run_success "$OCKAM" lease --identity m2 --project-path "$PROJECT_JSON_PATH" create

  # m3 can't
  run_success "$OCKAM" lease --identity m3 --project-path "$PROJECT_JSON_PATH" create
  assert_failure

  unset OCKAM_HOME
  run_success "$OCKAM" project addon configure influxdb --org-id "${INFLUXDB_ORG_ID}" --token "${INFLUXDB_TOKEN}" --endpoint-url "${INFLUXDB_ENDPOINT}" --max-ttl 60 --permissions "${INFLUXDB_PERMISSIONS}" --user-access-role '(= subject.service "sensor")'

  sleep 30 #FIXME  workaround, project not yet ready after configuring addon

  export OCKAM_HOME=/tmp/ockam
  # m1 can use the lease manager (it has a service=sensor attribute attested by authority)
  run_success "$OCKAM" lease --identity m1 --project-path "$PROJECT_JSON_PATH" create

  # m2 can't use the  lease manager now (it doesn't have a service=sensor attribute attested by authority)
  run_failure "$OCKAM" lease --identity m2 --project-path "$PROJECT_JSON_PATH" create
}
