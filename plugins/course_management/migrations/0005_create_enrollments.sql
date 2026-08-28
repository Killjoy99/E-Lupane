CREATE TABLE enrollments
(
    id UUID PRIMARY KEY,

    offering_id UUID
        REFERENCES course_offerings(id),

    student_id UUID NOT NULL,

    enrolled_at TIMESTAMPTZ DEFAULT NOW(),

    status VARCHAR(20)
);