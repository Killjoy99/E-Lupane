-- Idempotent seed for the User Support module.
-- Adds the `support` role and a demo support staff account (username `support`,
-- password `support123`, role `support`).
--
-- Safe to re-run. If the auth plugin has not created its tables yet (first run
-- on a fresh database), this block reports a NOTICE and exits without error;
-- re-running serve.sh once the gateway has started auth will complete the seed.
DO $$
DECLARE
    v_role_id uuid;
    v_user_id uuid;
BEGIN
    IF to_regclass('public.roles') IS NULL
       OR to_regclass('public.users') IS NULL
       OR to_regclass('public.user_roles') IS NULL THEN
        RAISE NOTICE 'auth schema not present yet; skipping support seed';
        RETURN;
    END IF;

    INSERT INTO roles (name, description)
    VALUES ('support', 'User support staff - triages help-desk tickets and authors FAQ')
    ON CONFLICT (name) DO NOTHING;

    SELECT id INTO v_role_id FROM roles WHERE name = 'support';

    INSERT INTO users (username, email, password_hash, full_name, active)
    VALUES ('support', 'support@elupane.local',
            '$2b$10$8UXMvvKuZ4IKjs2o2EjQMemb43JNsliX5IvRxSUI4K1NMn/Mobbi2',
            'Support Desk', TRUE)
    ON CONFLICT (username) DO NOTHING;

    SELECT id INTO v_user_id FROM users WHERE username = 'support';

    INSERT INTO user_roles (user_id, role_id) VALUES (v_user_id, v_role_id)
    ON CONFLICT DO NOTHING;

    RAISE NOTICE 'support role + demo account ensured';
END $$;
