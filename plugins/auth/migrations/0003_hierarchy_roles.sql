-- Auth plugin v3: academic hierarchy roles — dean and hod.
--
-- * dean  = faculty head (heads a Faculty); oversees departments.
-- * hod    = departmental chairperson (heads a Department); organises
--             the department's subjects, courses and lecturers.
--
-- Lecturer remains the teaching role under the department. The permission
-- grants mirror lecturer for hod (department ops) and grant read-across
-- faculties/departments for dean (faculty oversight).

INSERT INTO roles (name, description) VALUES
    ('dean', 'Faculty head — oversees faculties, departments and academic programmes'),
    ('hod',  'Departmental chairperson — heads a department and its lecturers')
ON CONFLICT (name) DO NOTHING;

-- Permissions needed by the academic hierarchy (administrative + read-only bolts
-- on top of administrative core may be granted selectively by a super admin).
INSERT INTO permissions (name, description) VALUES
    ('academics.faculty.read',     'View faculties the user oversees'),
    ('academics.department.read',   'View departments the user oversees'),
    ('academics.department.manage', 'Manage departments (create/edit heads)'),
    ('academics.faculty.manage',   'Manage faculties (dean appointment)'),
    ('subjects.manage',             'Create, edit and publish subjects in courses'),
    ('subjects.invite',               'Create invite links to subjects'),
    ('students.manage',              'View and manage students in the department')
ON CONFLICT (name) DO NOTHING;

-- HoD: department-level rights (what lecturers can do, plus department/subject management).
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'hod'
  AND p.name IN (
      'users.read', 'roles.assign', 'courses.manage', 'enrollments.manage',
      'academics.faculty.read', 'academics.department.read',
      'academics.department.manage', 'subjects.manage', 'subjects.invite',
      'students.manage', 'content.manage', 'assignments.manage', 'live.manage'
  )
ON CONFLICT DO NOTHING;

-- Dean: faculty-level rights (everything HoD can do, plus faculty management.)
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'dean'
  AND p.name IN (
      'users.read', 'roles.assign', 'courses.manage', 'enrollments.manage',
      'academics.faculty.read', 'academics.department.read',
      'academics.faculty.manage', 'academics.department.manage',
      'subjects.manage', 'subjects.invite', 'students.manage',
      'content.manage', 'assignments.manage', 'live.manage', 'analytics.read'
  )
ON CONFLICT DO NOTHING;