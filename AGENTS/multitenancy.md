Convert the LMS into an optional multi-tenant architecture.

A tenant represents an institution or organization.

Support:

Tenant
  → Faculties
  → Departments
  → Programs
  → Users
  → Courses
  → Enrollments
  → Academic data

Every tenant-owned resource must be isolated.

Prevent cross-tenant data access.

Implement tenant-aware authorization.

Determine whether the existing architecture should use:

- Shared database/shared schema
- Shared database/separate schema
- Separate databases

Choose the simplest architecture that provides strong isolation and future scalability.

Add automated cross-tenant security tests.