---
name: plan-reader
description: Guide for reading and understanding plans in tmp/active directory. Triggers on: tmp/active, active plan, continue task, resume work, handoff notes, task progress, dev-docs, picking up work, checking progress
---

# Plan Reader

## Purpose

Helps Claude properly load and understand existing project plans from `tmp/active/{project-name}/` to continue work seamlessly across sessions.

## When to Use

- Starting a new session on an existing project
- Checking progress on a multi-phase implementation
- Understanding what was done in previous sessions
- Finding the next tasks to work on
- Resuming work after context compaction

## Three-File Structure

Plans created by `dev-docs` follow a consistent three-file structure:

### 1. {project}-tasks.md (Read First)

This is your **quick status check** - read it first to understand current state.

Contains:

- **Progress table** - Quick overview of phase completion (X/Y tasks per phase)
- **Session notes** - What was done recently with timestamps
- **Handoff notes** - Exact state for continuation
- **Task checklists** - `[x]` done, `[ ]` pending
- **Verification commands** - How to test completed work

### 2. {project}-context.md (Read Second)

This provides the **decisions and dependencies** needed to understand the codebase state.

Contains:

- **Key files table** - Status of each file (created/modified/pending)
- **Critical decisions** - Rationale for architectural choices
- **Dependencies** - External requirements and versions
- **Integration points** - How components connect
- **Implementation notes** - Important technical details

### 3. {project}-plan.md (Reference as Needed)

This is the **full strategic plan** - reference when you need phase details.

Contains:

- **Executive summary** - Overall goals and scope
- **Phases** - Ordered implementation steps with details
- **Acceptance criteria** - Definition of done for the project
- **Risk assessment** - Potential issues and mitigations
- **Timeline** - Phase ordering and dependencies

## Reading Workflow

Follow this order for efficient context loading:

```
1. List tmp/active/ to find active projects
   └─> ls tmp/active/

2. Read {project}-tasks.md for current status
   └─> Quick overview of what's done and what's next
   └─> Check handoff notes for exact continuation point

3. Read {project}-context.md for decisions and state
   └─> Understand file statuses
   └─> Review critical decisions made

4. Reference {project}-plan.md only when needed
   └─> Full phase details
   └─> Acceptance criteria verification
```

## Key Patterns to Look For

### Progress Summary Table

```markdown
| Phase   | Status      | Progress |
| ------- | ----------- | -------- |
| Phase 1 | Complete    | 5/5      |
| Phase 2 | In Progress | 3/7      |
| Phase 3 | Pending     | 0/4      |
```

### Handoff Notes Section

```markdown
## Handoff Notes

**Last Session**: 2024-01-15
**Stopped At**: Implementing fixture builder validation
**Next Steps**:

1. Add error handling for circular references
2. Test with complex schemas
```

### Session Notes

```markdown
## Session Notes

### Session 3 (2024-01-15)

- Completed createFixture core implementation
- Added tests for basic functionality
- Discovered edge case with nullable fields
```

### Next Immediate Steps

```markdown
## Next Immediate Steps

1. [ ] Fix nullable field handling in createFixture
2. [ ] Add integration tests
3. [ ] Update documentation
```

### Verification Commands

````markdown
## Verification Commands

```bash
# Run tests for completed features
bun test packages/search-index

# Type check
bun typecheck
```
````

```

## Example Session Start

When resuming work on a project:

```

User: "Continue work on search-index"

Claude should:

1. ls tmp/active/
2. Read tmp/active/search-index/search-index-tasks.md
3. Summarize current status to user
4. Read tmp/active/search-index/search-index-context.md
5. Begin work on next pending task from handoff notes

```

## Integration with Other Skills

- **dev-docs**: Creates these plan files initially
- **dev-docs-update**: Updates plans during/after work sessions
- **plan-reader**: Helps READ and understand existing plans (this skill)

## Tips

- Always check the handoff notes first - they contain the exact continuation point
- The progress table gives you a quick status without reading everything
- Session notes help understand the reasoning behind recent changes
- Verification commands let you validate work is still passing
```
