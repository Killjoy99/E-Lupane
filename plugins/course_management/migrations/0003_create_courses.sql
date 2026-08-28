CREATE TABLE courses
(
    id UUID PRIMARY KEY,

    programme_id UUID
        REFERENCES programmes(id),

    course_code VARCHAR(20) UNIQUE NOT NULL,

    course_name VARCHAR(255) NOT NULL,

    description TEXT,

    credits INTEGER NOT NULL,

    level INTEGER,

    semester INTEGER,

    active BOOLEAN DEFAULT TRUE,

    created_at TIMESTAMPTZ DEFAULT NOW(),

    updated_at TIMESTAMPTZ DEFAULT NOW()
);