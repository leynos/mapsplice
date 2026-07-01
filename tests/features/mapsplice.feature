Feature: Mapsplice roadmap editing

  Scenario: Append emits the rewritten roadmap to stdout
    Given the target roadmap with two phases
    And the phase fragment roadmap
    When I append the phase fragment
    Then the command succeeds
    And stdout contains the appended phase as phase 3
    And the target file remains unchanged

  Scenario: Insert before a phase renumbers later phases and dependencies
    Given the target roadmap with two phases
    And the phase fragment roadmap
    When I insert the phase fragment before phase 2
    Then the command succeeds
    And stdout renumbers phase two to phase 3 and rewrites its dependency

  Scenario: Insert after a task renumbers later tasks within the step
    Given the target roadmap with one step and two tasks
    And the task fragment roadmap
    When I insert the task fragment after task 1.1.1
    Then the command succeeds
    And stdout renumbers the old second task to 1.1.3

  Scenario: Delete removes an addressed phase and rewrites downstream identifiers
    Given the target roadmap with three phases
    When I delete phase 2
    Then the command succeeds
    And stdout removes phase 2 and rewrites the remaining dependency to 2.1.1

  Scenario: Delete preserves scoped_reference incidental numbers while rewriting Requires dependencies
    Given the target roadmap with scoped reference text
    When I delete phase 1
    Then the command succeeds
    And stdout preserves scoped_reference incidental numbers and rewrites Requires dependencies

  Scenario: Replace swaps a phase with multiple phases from a fragment file
    Given the target roadmap with two phases
    And the replacement fragment roadmap
    When I replace phase 2 with the replacement fragment
    Then the command succeeds
    And stdout contains replacement phases 2 and 3

  Scenario: In-place mode rewrites the target file and emits no roadmap body
    Given the target roadmap with two phases
    When I delete phase 1 in place
    Then the command succeeds
    And stdout is empty
    And the target file now starts with phase 1 titled Phase two

  Scenario: Level mismatch returns a clear failure
    Given the target roadmap with two phases
    And the task fragment roadmap
    When I try to insert the mismatched fragment before phase 2
    Then the command fails
    And stderr mentions the phase versus task mismatch

  Scenario: Missing anchor returns a clear failure
    Given the target roadmap with two phases
    When I try to delete missing phase 99
    Then the command fails
    And stderr mentions that anchor 99 was not found
