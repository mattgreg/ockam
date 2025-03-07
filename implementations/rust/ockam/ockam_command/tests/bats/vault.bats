#!/bin/bash

# ===== SETUP

setup() {
  load load/base.bash
  load_bats_ext
  setup_home_dir
}

teardown() {
  teardown_home_dir
}

# ===== TESTS

@test "vault - create and check show/list output" {
  v1=$(random_str)
  run_success "$OCKAM" vault create "${v1}"

  run_success "$OCKAM" vault show "${v1}"
  assert_output --partial "Name: ${v1}"
  assert_output --partial "Type: OCKAM"

  v2=$(random_str)
  run_success "$OCKAM" vault create "${v2}"

  run_success "$OCKAM" vault show "${v2}"
  assert_output --partial "Name: ${v2}"
  assert_output --partial "Type: OCKAM"

  run_success "$OCKAM" vault list
  assert_output --partial "Vault ${v1}"
  assert_output --partial "Type OCKAM"
  assert_output --partial "Vault ${v2}"
  assert_output --partial "Type OCKAM"
}

@test "vault - CRUD" {
  # Create with random name
  run_success "$OCKAM" vault create

  # Create with specific name
  v=$(random_str)

  run_success "$OCKAM" vault create "${v}"
  run_success "$OCKAM" vault delete "${v}" --yes
  run_failure "$OCKAM" vault show "${v}"

  # Delete vault and leave identities untouched
  v=$(random_str)
  i=$(random_str)

  run_success "$OCKAM" vault create "${v}"
  run_success "$OCKAM" identity create "${i}" --vault "${v}"
  run_success "$OCKAM" vault delete "${v}" --yes
  run_failure "$OCKAM" vault show "${v}"
  run_success "$OCKAM" identity show "${i}"
}
