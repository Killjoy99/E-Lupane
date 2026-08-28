CREATE TABLE course_offerings
(
    id UUID PRIMARY KEY,

    course_id UUID
        REFERENCES courses(id),

    lecturer_id UUID NOT NULL,

    academic_year VARCHAR(20),

    semester INTEGER,

    active BOOLEAN DEFAULT TRUE,

    created_at TIMESTAMPTZ DEFAULT NOW()
);