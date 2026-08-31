# Assignments Plugin (`com.elupane.assignments`)

Assignment lifecycle, submissions, deadlines, and grading for E-Lupane.

## Capabilities / operations

Every request payload must include a JWT issued by the auth plugin:
`{ "token": "<jwt>", ...fields }`.

| Op | Who | Description |
|----|-----|-------------|
| `assignment.create` | instructor | Create an assignment (title, instructions, course, deadline, max marks, allowed attempts, submission type, late policy + penalty) |
| `assignment.update` | instructor | Edit any assignment field |
| `assignment.delete` | instructor | Delete an assignment (cascades resources/submissions/grades) |
| `assignment.get` / `assignment.list` | any user | View assignment(s) |
| `assignment.resource.attach` | instructor | Attach a resource file (base64 content) |
| `assignment.resource.remove` | instructor | Remove an attached resource |
| `assignment.resource.download` | instructor / student | Download a resource file (base64 in the reply) |
| `submission.submit` | student | Submit text / file(s) / link(s) (a resubmission = a new attempt) |
| `submission.status` | student / instructor | Latest submission for the caller (or all, for instructors) |
| `submission.download` | owner / instructor | Download a submitted file |
| `grade.create` | instructor | Grade a submission (marks, comments, feedback); late penalty applied automatically |
| `grade.get` | owner / instructor | Current grade |
| `grade.history` | owner / instructor | Append-only grading history |
| `rubric.create` / `rubric.list` / `rubric.get` | create: instructor | Rubrics with JSON criteria |

## Deadline enforcement & late handling

* A submission after `deadline` is marked `is_late = true`.
* `late_submission_policy`:
  * `disallowed` — late submissions are rejected outright.
  * `allowed` — accepted, no penalty.
  * `allowed_with_penalty` (default) — accepted; grading deducts
    `late_penalty_percent` (a % of **max_marks**) from the raw marks, never
    below zero. Final marks are stored alongside the raw marks and both are
    recorded in the grade history.
* `allowed_attempts` caps the number of submissions a student may make.

## Authorization

The plugin verifies the caller's JWT locally using the shared `AUTH_JWT_SECRET`
(issued by the auth plugin). `admin`/`lecturer` are instructors; `student`
submits work. Students may only access their own submissions/grades/files;
instructors may access everything.

## Secure file uploads

* Files are stored under `ASSIGNMENTS_STORAGE_DIR` (default
  `data/assignments/<assignment-id>/`) with **server-generated random names**.
* User-supplied file names are kept only in the database; path-traversal names
  are rejected (`/`, `\`, `..`, NUL) and reads are re-verified to stay inside
  the assignment directory.
* Per-file size limit: `ASSIGNMENTS_MAX_FILE_BYTES` (default 25 MiB).
* sha256 content hashes are stored for integrity checks.

## Schema

`migrations/0002_assignments.sql` (version `0002` — the auth plugin owns `0001`
in the shared migrations table): `assignments`, `assignment_resources`,
`submissions`, `submission_files`, `submission_links`, `grades`,
`grade_history`, `rubrics`.

## Tests

```sh
cargo test -p assignments
```

17 unit tests cover: deadline classification, on-time / late submission
admission, `disallowed` late policy rejection, attempt-limit enforcement,
late-penalty maths (including the zero floor), submission-type content
validation, JWT round-trips + role checks, filename sanitisation, and the
secure storage round-trip.
