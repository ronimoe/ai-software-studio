# Exploration: MCP Integration Options

## Status

Future Exploration

## Question

Should AI Software Studio expose an MCP server so agents can interact with the app more directly?

## Context

The initial MVP controls agents through local CLI adapters.

Later, a stronger integration could allow agents to call structured tools such as:

- get_task
- get_acceptance_criteria
- report_plan
- request_approval
- submit_evidence
- update_status
- attach_artifact

## Potential MCP Tools

```text
get_current_task()
get_acceptance_criteria(task_id)
get_constraints(task_id)
report_plan(task_id, plan)
request_human_approval(task_id, reason)
submit_artifact(task_id, artifact)
mark_criterion_satisfied(task_id, criterion_id, evidence)
```

## Benefits

- Better structured communication with agents
- Less reliance on parsing terminal output
- Stronger workflow integration
- Better future multi-agent support

## Risks

- Adds complexity
- Depends on engine support
- May not be needed for MVP
- Need clear permission model

## Recommended Direction

Do not build MCP in MVP.

Keep it as a future integration layer after the CLI adapter workflow is validated.

## Decision Trigger

Revisit after the first working local agent workflow is stable.
