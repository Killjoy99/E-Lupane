CREATE TABLE programmes
(
    id UUID PRIMARY KEY,

    department_id UUID NOT NULL
        REFERENCES departments(id),

    code VARCHAR(30) UNIQUE NOT NULL,

    name VARCHAR(255) NOT NULL,

    duration INTEGER NOT NULL,

    created_at TIMESTAMPTZ DEFAULT NOW(),

    updated_at TIMESTAMPTZ DEFAULT NOW()
);