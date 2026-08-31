Implement the academic structure required for a university/college LMS.

Create:

- Institutions
- Faculties
- Departments
- Programs
- Academic years
- Semesters
- Classes/cohorts
- Courses
- Course offerings

Support relationships such as:

Institution
  → Faculty
    → Department
      → Program
        → Course

Academic Year
  → Semester
    → Course Offering
      → Instructor
      → Students

Implement CRUD APIs and administrator interfaces.

Add validation for invalid relationships and duplicate records.

Add tests.