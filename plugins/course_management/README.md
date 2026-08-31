# Course Management Plugin

Course lifecycle management for E-Lupane: courses, instructors, statuses,
publishing, duplication, and ownership enforcement.

## Operations

| Op | Who | Description |
|---|---|---|
| `course.create` | instructor/admin | Create a course (starts as `draft`) |
| `course.get` | any authenticated | View a course (published for everyone; non-public only for managers) |
| `course.list` | any authenticated | Role-aware listing with `status` / `mine` / `search` filters |
| `course.update` | owner/managers | Edit course fields |
| `course.delete` | owner/admin | Hard-delete a course |
| `course.archive` | owner/managers | Soft-delete (status → `archived`) |
| `course.duplicate` | owner/managers | Copy as a fresh draft (`Copy of …`, unique code) |
| `course.preview` | owner/managers | Read a course in any status |
| `course.set_status` | owner/managers | Transition status (validated) |
| `course.publish` / `course.unpublish` | owner/managers | Publish / take down |
| `course.submit_review` | owner/managers | Send for review (`pending_review`) |
| `course.instructor.add` / `.remove` | owner/managers | Authorize/revoke co-instructors |

## Course fields

Title, code, description, short description, thumbnail, banner, category,
subject, level, language, difficulty, duration, learning objectives, target
audience, prerequisites, tags, instructors, and status.

## Statuses & transitions

```
draft ──────────► pending_review ──► published ──► unpublished ──► archived
  │                    │  ▲              │  ▲          │  ▲           │
  └────► archived      └──┴── draft      └──┴──────────┘  └──────────► draft
```

`draft`, `pending_review`, `published`, `unpublished`, `archived`.
Admins may force any transition; instructors follow the state machine.

## Ownership

* Only instructors (admin/lecturer) can create courses.
* The **owner** and users listed in `course_instructors` may manage a course.
* Only the owner (or an admin) may hard-delete.
* Students can only see `published` courses.

## Linking with Assignments and Live Lessons

The assignments plugin stores `course_id` on each assignment. This plugin's
migration promotes that column into a real foreign key
(`fk_assignments_course`), and the assignments service validates that an
instructor may only attach assignments to courses they own or manage.

The live classes plugin also ties each scheduled lesson/session to a course via
`live_sessions.course_id`, and this migration adds `fk_live_sessions_course` so
course-scoped learning objects cannot drift outside the course registry.

## Storage

* `migrations/0003_courses.sql` — applied as idempotent raw SQL (the auth
  plugin owns `0001` and assignments `0002` in the shared `_sqlx_migrations`
  table, so `sqlx::migrate!` cannot be used).

## Tests

`cargo test -p course_management` runs the pure-logic unit tests: status
transition validation, course-code validation, duplicate-title generation,
difficulty normalization, and the ownership rules (`can_manage` / `can_view`).
