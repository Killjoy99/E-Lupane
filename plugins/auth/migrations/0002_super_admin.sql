-- Auth plugin v2: super admin role + management permissions.
--
-- Adds a `super_admin` role above `admin` and extends the permission catalog
-- used by the admin permission matrix UI. `super_admin` receives every
-- permission, and future permissions added here are auto-granted to it.

INSERT INTO roles (name, description) VALUES
    ('super_admin', 'Platform owner — all permissions, all dashboards')
ON CONFLICT (name) DO NOTHING;

INSERT INTO permissions (name, description) VALUES
    ('users.delete',      'Delete user accounts'),
    ('roles.manage',      'Create roles and edit role permissions'),
    ('content.manage',    'Manage modules, lessons and files'),
    ('assignments.manage','Create and grade assignments'),
    ('live.manage',       'Schedule and run live sessions'),
    ('analytics.read',    'View platform analytics'),
    ('plugins.manage',    'Enable/disable plugins and approve keys'),
    ('institutions.manage','Manage institutions')
ON CONFLICT (name) DO NOTHING;

-- Admin gets everything except the super-admin-only extras? No: admin keeps
-- full coverage (it is the operational admin), super_admin is the owner tier.
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name IN ('super_admin', 'admin')
ON CONFLICT DO NOTHING;
