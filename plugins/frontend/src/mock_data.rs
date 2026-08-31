//! Mock/demo data used by the role pages (kept out of the dashboard shell).

// Mock data
// ---------------------------------------------------------------------------

pub(crate) struct Course {
    pub code: &'static str,
    pub name: &'static str,
    pub instructor: &'static str,
    pub progress: f64,
    pub students: u32,
    pub pending: u32,
    pub tone: &'static str,
    pub icon: &'static str,
}

pub(crate) const COURSES: &[Course] = &[
    Course { code: "CS101", name: "Intro to Programming (Python)", instructor: "Dr. N. Dlamini", progress: 72.0, students: 38, pending: 5, tone: "teal", icon: "code" },
    Course { code: "DB201", name: "Database Systems (PostgreSQL)", instructor: "Prof. T. Moyo", progress: 45.0, students: 42, pending: 3, tone: "indigo", icon: "database" },
    Course { code: "WEB302", name: "Full-Stack Web (Rust + Dioxus)", instructor: "Dr. K. Ncube", progress: 88.0, students: 25, pending: 2, tone: "green", icon: "web" },
    Course { code: "MATH150", name: "Linear Algebra", instructor: "Prof. L. Sibanda", progress: 34.0, students: 51, pending: 7, tone: "amber", icon: "functions" },
    Course { code: "AI410", name: "Machine Learning", instructor: "Dr. P. Khumalo", progress: 12.0, students: 18, pending: 0, tone: "purple", icon: "smart_toy" },
];

pub(crate) struct Assignment {
    pub title: &'static str,
    pub course: &'static str,
    pub due: &'static str,
    pub status: &'static str,
    pub grade: Option<&'static str>,
}

pub(crate) const ASSIGNMENTS: &[Assignment] = &[
    Assignment { title: "Sorting algorithms — Part 2", course: "CS101", due: "Today, 23:59", status: "pending", grade: None },
    Assignment { title: "Normalisation exercise", course: "DB201", due: "Fri, 17:00", status: "submitted", grade: None },
    Assignment { title: "Dioxus fullstack mini-project", course: "WEB302", due: "Mon, 23:59", status: "graded", grade: Some("87%") },
    Assignment { title: "Matrix multiplication", course: "MATH150", due: "Next Tue, 23:59", status: "pending", grade: None },
];

pub(crate) struct Quiz {
    pub title: &'static str,
    pub course: &'static str,
    pub when: &'static str,
    pub info: &'static str,
}

pub(crate) const QUIZZES: &[Quiz] = &[
    Quiz { title: "Week 4 quiz · Functions", course: "CS101", when: "Thu 14:00", info: "10 questions · 15 min" },
    Quiz { title: "SQL joins quiz", course: "DB201", when: "Fri 09:00", info: "12 questions · 20 min" },
];

pub(crate) struct Announcement {
    pub title: &'static str,
    pub body: &'static str,
    pub author: &'static str,
    pub time: &'static str,
}

pub(crate) const ANNOUNCEMENTS: &[Announcement] = &[
    Announcement { title: "Examination timetable published", body: "The May/June exam timetable is now available on the registrar's portal.", author: "Office of the Registrar", time: "2h ago" },
    Announcement { title: "Library extended hours during exams", body: "All campus libraries stay open until 22:00 from next Monday.", author: "Library Services", time: "1d ago" },
    Announcement { title: "Plugin SDK workshop", body: "Hands-on session on building E-Lupane plugins with the Rust & Python SDKs.", author: "Platform Team", time: "3d ago" },
];

pub(crate) struct StudentRow {
    pub name: &'static str,
    pub initials: &'static str,
    pub course: &'static str,
    pub grade: &'static str,
    pub status: &'static str,
}

pub(crate) const ROSTER: &[StudentRow] = &[
    StudentRow { name: "Aisha Patel", initials: "AP", course: "CS101", grade: "93%", status: "on-track" },
    StudentRow { name: "Sipho Ndlovu", initials: "SN", course: "CS101", grade: "82%", status: "on-track" },
    StudentRow { name: "Naledi Mokoena", initials: "NM", course: "CS101", grade: "74%", status: "on-track" },
    StudentRow { name: "Tendai Moyo", initials: "TM", course: "CS101", grade: "68%", status: "on-track" },
    StudentRow { name: "Kgotso Molefe", initials: "KM", course: "CS101", grade: "61%", status: "at-risk" },
    StudentRow { name: "Lerato Dube", initials: "LD", course: "CS101", grade: "55%", status: "at-risk" },
];

pub(crate) struct Submission {
    pub name: &'static str,
    pub item: &'static str,
    pub course: &'static str,
    pub time: &'static str,
    pub state: &'static str,
}

pub(crate) const SUBMISSIONS: &[Submission] = &[
    Submission { name: "Aisha Patel", item: "Sorting algorithms — Part 2", course: "CS101", time: "12 min ago", state: "new" },
    Submission { name: "Tendai Moyo", item: "Normalisation exercise", course: "DB201", time: "1h ago", state: "new" },
    Submission { name: "Kgotso Molefe", item: "Sorting algorithms — Part 2", course: "CS101", time: "2h ago", state: "flagged" },
    Submission { name: "Naledi Mokoena", item: "Dioxus mini-project", course: "WEB302", time: "Yesterday", state: "graded" },
];

pub(crate) struct Deadline {
    pub title: &'static str,
    pub course: &'static str,
    pub when: &'static str,
    pub kind: &'static str,
}

pub(crate) const DEADLINES: &[Deadline] = &[
    Deadline { title: "Grade sorting algorithms — Part 2", course: "CS101", when: "Today, 23:59", kind: "grading" },
    Deadline { title: "Publish week 5 lab brief", course: "CS101", when: "Thu 12:00", kind: "coursework" },
    Deadline { title: "Finalise DB201 marks", course: "DB201", when: "Fri 17:00", kind: "grading" },
];


pub(crate) struct PluginRow {
    pub name: &'static str,
    pub lang: &'static str,
    pub version: &'static str,
    pub status: &'static str,
    pub signed: bool,
    pub port: u16,
}

pub(crate) const PLUGINS: &[PluginRow] = &[
    PluginRow { name: "institution_management", lang: "Rust", version: "0.1.0", status: "running", signed: true, port: 9101 },
    PluginRow { name: "course_management", lang: "Rust", version: "0.1.0", status: "running", signed: true, port: 9102 },
    PluginRow { name: "assignments", lang: "Rust", version: "0.1.0", status: "running", signed: true, port: 9103 },
    PluginRow { name: "quizzes", lang: "Python", version: "0.1.0", status: "running", signed: true, port: 9104 },
    PluginRow { name: "analytics", lang: "Rust", version: "0.2.1", status: "degraded", signed: true, port: 9105 },
    PluginRow { name: "notifications", lang: "Python", version: "0.1.0", status: "running", signed: true, port: 9106 },
];

pub(crate) struct Service {
    pub name: &'static str,
    pub status: &'static str,
    pub detail: &'static str,
}

pub(crate) const SERVICES: &[Service] = &[
    Service { name: "Gateway", status: "ok", detail: "actix-web · port 8080" },
    Service { name: "PostgreSQL", status: "ok", detail: "elupane · 12 connections" },
    Service { name: "Redis", status: "ok", detail: "event bus & cache" },
    Service { name: "Event bus", status: "ok", detail: "0 failed publishes" },
    Service { name: "Media service", status: "ok", detail: "S3 storage · 84% free" },
    Service { name: "Search service", status: "warn", detail: "index lag · 2m behind" },
];

pub(crate) struct Sdk {
    pub lang: &'static str,
    pub version: &'static str,
    pub status: &'static str,
    pub note: &'static str,
}

pub(crate) const SDKS: &[Sdk] = &[
    Sdk { lang: "Rust", version: "0.1.0", status: "available", note: "Native, first-class support" },
    Sdk { lang: "Python", version: "0.1.0", status: "available", note: "Built with maturin" },
    Sdk { lang: "WebAssembly", version: "0.1.0-beta", status: "review", note: "Sandboxed WASM plugins" },
    Sdk { lang: "PHP", version: "—", status: "pending", note: "Structured shared objects" },
    Sdk { lang: "C#", version: "—", status: "pending", note: "On the roadmap" },
];

pub(crate) struct UserRow {
    pub name: &'static str,
    pub email: &'static str,
    pub role: &'static str,
    pub institution: &'static str,
    pub status: &'static str,
}

pub(crate) const USERS: &[UserRow] = &[
    UserRow { name: "Aisha Patel", email: "aisha.patel@lupane.ac.zw", role: "Student", institution: "Lupane State University", status: "active" },
    UserRow { name: "Dr. Nkosana Dlamini", email: "n.dlamini@lupane.ac.zw", role: "Lecturer", institution: "Lupane State University", status: "active" },
    UserRow { name: "Lerato Molefe", email: "lerato@elupane.dev", role: "Developer", institution: "Platform", status: "active" },
    UserRow { name: "Kgotso Molefe", email: "kgotso.m@bit.ac.zw", role: "Student", institution: "Bulawayo Institute of Tech", status: "active" },
    UserRow { name: "Prof. L. Sibanda", email: "l.sibanda@hsc.ac.zw", role: "Lecturer", institution: "Harare School of Computing", status: "suspended" },
    UserRow { name: "Simphiwe Khumalo", email: "skhumalo@elupane.dev", role: "Admin", institution: "Platform", status: "active" },
];

pub(crate) struct Institution {
    pub name: &'static str,
    pub students: u32,
    pub courses: u32,
    pub tier: &'static str,
}

pub(crate) const INSTITUTIONS: &[Institution] = &[
    Institution { name: "Lupane State University", students: 612, courses: 34, tier: "Enterprise" },
    Institution { name: "Bulawayo Institute of Technology", students: 284, courses: 19, tier: "Standard" },
    Institution { name: "Harare School of Computing", students: 198, courses: 12, tier: "Standard" },
    Institution { name: "Midlands Innovation College", students: 154, courses: 9, tier: "Starter" },
];

pub(crate) struct Approval {
    pub plugin: &'static str,
    pub lang: &'static str,
    pub requester: &'static str,
    pub key: &'static str,
}

pub(crate) const APPROVALS: &[Approval] = &[
    Approval { plugin: "analytics v0.3.0", lang: "Rust", requester: "L. Molefe", key: "lupane-ops-7f3a" },
    Approval { plugin: "attendance-tracker", lang: "Python", requester: "M. Ncube", key: "lupane-dev-91b2" },
];

pub(crate) struct Activity {
    pub icon: &'static str,
    pub text: &'static str,
    pub time: &'static str,
    pub tone: &'static str,
}

pub(crate) const ACTIVITY: &[Activity] = &[
    Activity { icon: "verified", text: "Plugin `analytics v0.3.0` signed & approved", time: "4h ago", tone: "teal" },
    Activity { icon: "upload_file", text: "Bulk import — 214 students added", time: "1d ago", tone: "indigo" },
    Activity { icon: "domain", text: "Harare School of Computing onboarded", time: "2d ago", tone: "green" },
    Activity { icon: "key", text: "New signing key issued to developer M. Ncube", time: "3d ago", tone: "amber" },
];

pub(crate) const ROLE_SPLIT: &[(&str, u32, &str)] = &[
    ("Students", 1015, "teal"),
    ("Lecturers", 126, "indigo"),
    ("Developers", 22, "green"),
    ("Admins", 5, "purple"),
];

// --- Student messages -------------------------------------------------------

pub(crate) struct Msg {
    pub text: &'static str,
    pub sent: bool,
    pub time: &'static str,
}

pub(crate) struct Conversation {
    pub person: &'static str,
    pub role: &'static str,
    pub initials: &'static str,
    pub tone: &'static str,
    pub preview: &'static str,
    pub time: &'static str,
    pub unread: u32,
    pub online: bool,
    pub thread: &'static [Msg],
}

pub(crate) const CONVERSATIONS: &[Conversation] = &[
    Conversation {
        person: "Dr. Nkosana Dlamini",
        role: "Lecturer · CS101",
        initials: "ND",
        tone: "indigo",
        preview: "Your submission has been graded — great work on the sorting algorithms!",
        time: "09:41",
        unread: 2,
        online: true,
        thread: &[
            Msg { text: "Hi Thandiwe, I noticed your submission for 'Sorting algorithms — Part 2' came in just before the deadline.", sent: false, time: "09:20" },
            Msg { text: "I've finished marking it. You scored 87% — one of the best in the class.", sent: false, time: "09:38" },
            Msg { text: "Thank you so much, Dr. Dlamini! I really enjoyed the divide-and-conquer section.", sent: true, time: "09:42" },
            Msg { text: "Keep it up. Your submission has been graded — great work on the sorting algorithms!", sent: false, time: "09:41" },
        ],
    },
    Conversation {
        person: "Aisha Patel",
        role: "Classmate · CS101",
        initials: "AP",
        tone: "teal",
        preview: "Do you want to form a study group for the DB201 quiz on Friday?",
        time: "Yesterday",
        unread: 0,
        online: true,
        thread: &[
            Msg { text: "Hey! Did you finish the SQL joins worksheet?", sent: false, time: "Yesterday, 14:02" },
            Msg { text: "Almost — the nested joins are still tripping me up.", sent: true, time: "Yesterday, 14:15" },
            Msg { text: "Do you want to form a study group for the DB201 quiz on Friday?", sent: false, time: "Yesterday, 14:22" },
        ],
    },
    Conversation {
        person: "Prof. L. Sibanda",
        role: "Lecturer · MATH150",
        initials: "LS",
        tone: "amber",
        preview: "Problem set 4 solutions are now available on the course page.",
        time: "Mon",
        unread: 0,
        online: false,
        thread: &[
            Msg { text: "Problem set 4 solutions are now available on the course page.", sent: false, time: "Mon, 10:00" },
            Msg { text: "Any past paper resources you'd recommend for the test?", sent: true, time: "Mon, 11:30" },
        ],
    },
    Conversation {
        person: "Study Support",
        role: "Library Services",
        initials: "SS",
        tone: "purple",
        preview: "Your booking for the group study room (2PM Fri) is confirmed.",
        time: "Sun",
        unread: 0,
        online: false,
        thread: &[
            Msg { text: "Your booking for the group study room (2PM Fri) is confirmed.", sent: false, time: "Sun, 18:00" },
        ],
    },
];

// --- Charts -----------------------------------------------------------------

pub(crate) const WEEKLY_ACTIVITY: &[(&str, f64, &str)] = &[
    ("Mon", 62.0, "teal"),
    ("Tue", 48.0, "teal"),
    ("Wed", 74.0, "teal"),
    ("Thu", 90.0, "teal"),
    ("Fri", 55.0, "teal"),
    ("Sat", 30.0, "teal"),
    ("Sun", 42.0, "teal"),
];

pub(crate) const ENGAGEMENT: &[(&str, f64, &str)] = &[
    ("CS101", 92.0, "teal"),
    ("DB201", 84.0, "indigo"),
    ("WEB302", 78.0, "green"),
    ("MATH150", 65.0, "amber"),
    ("AI410", 58.0, "purple"),
];

pub(crate) const GRADE_DIST: &[(&str, f64, &str)] = &[
    ("A", 18.0, "green"),
    ("B", 32.0, "teal"),
    ("C", 28.0, "indigo"),
    ("D", 14.0, "amber"),
    ("F", 8.0, "purple"),
];

pub(crate) const WEEKLY_TREND: &[(&str, f64, &str)] = &[
    ("W1", 35.0, "indigo"),
    ("W2", 48.0, "indigo"),
    ("W3", 52.0, "indigo"),
    ("W4", 64.0, "indigo"),
    ("W5", 71.0, "indigo"),
    ("W6", 86.0, "indigo"),
];

// --- Student timetable ------------------------------------------------------

pub(crate) struct TtSlot {
    pub time: &'static str,
    pub course: &'static str,
    pub room: &'static str,
    pub live: bool,
}

pub(crate) struct TtDay {
    pub day: &'static str,
    pub slots: &'static [TtSlot],
}

pub(crate) const TIMETABLE: &[TtDay] = &[
    TtDay {
        day: "Mon",
        slots: &[
            TtSlot { time: "08:00", course: "CS101 Lecture", room: "Lab 2", live: false },
            TtSlot { time: "11:00", course: "MATH150 Tutorial", room: "T4", live: false },
        ],
    },
    TtDay {
        day: "Tue",
        slots: &[
            TtSlot { time: "09:00", course: "DB201 Lecture", room: "Hall A", live: false },
            TtSlot { time: "14:00", course: "WEB302 Lab", room: "Lab 5", live: true },
        ],
    },
    TtDay {
        day: "Wed",
        slots: &[
            TtSlot { time: "08:00", course: "CS101 Lab", room: "Lab 2", live: false },
            TtSlot { time: "13:00", course: "AI410 Seminar", room: "T2", live: false },
        ],
    },
    TtDay {
        day: "Thu",
        slots: &[
            TtSlot { time: "10:00", course: "DB201 Tutorial", room: "T1", live: false },
            TtSlot { time: "14:00", course: "CS101 Quiz", room: "Hall B", live: false },
        ],
    },
    TtDay {
        day: "Fri",
        slots: &[
            TtSlot { time: "09:00", course: "MATH150 Lecture", room: "Hall A", live: false },
            TtSlot { time: "12:00", course: "WEB302 Guest talk", room: "Auditorium", live: false },
        ],
    },
];

// --- Certificates -----------------------------------------------------------

pub(crate) struct Certificate {
    pub title: &'static str,
    pub issuer: &'static str,
    pub date: &'static str,
    pub credential: &'static str,
    pub tone: &'static str,
    pub icon: &'static str,
}

pub(crate) const CERTIFICATES: &[Certificate] = &[
    Certificate { title: "Rust Foundations", issuer: "E-Lupane Platform Team", date: "Issued Mar 2026", credential: "ELP-RUST-2026-0184", tone: "teal", icon: "verified" },
    Certificate { title: "Database Design & SQL", issuer: "Faculty of Computing", date: "Issued Jan 2026", credential: "ELP-DB-2026-0091", tone: "indigo", icon: "workspace_premium" },
    Certificate { title: "Web Development Bootcamp", issuer: "Faculty of Computing", date: "In progress · 64%", credential: "ELP-WEB-2026-0237", tone: "green", icon: "rocket_launch" },
];

// --- API docs ---------------------------------------------------------------

pub(crate) struct ApiEndpoint {
    pub method: &'static str,
    pub path: &'static str,
    pub desc: &'static str,
    pub auth: &'static str,
}

pub(crate) struct ApiGroup {
    pub name: &'static str,
    pub icon: &'static str,
    pub endpoints: &'static [ApiEndpoint],
}

pub(crate) const API_GROUPS: &[ApiGroup] = &[
    ApiGroup {
        name: "Gateway & Auth",
        icon: "lock_open",
        endpoints: &[
            ApiEndpoint { method: "POST", path: "/api/v1/auth/login", desc: "Exchange credentials for a JWT", auth: "Public" },
            ApiEndpoint { method: "POST", path: "/api/v1/auth/refresh", desc: "Rotate an expiring access token", auth: "Bearer" },
            ApiEndpoint { method: "GET", path: "/api/v1/users/me", desc: "Current user profile & roles", auth: "Bearer" },
            ApiEndpoint { method: "GET", path: "/api/health", desc: "Server & database liveness probe", auth: "Public" },
        ],
    },
    ApiGroup {
        name: "Courses & Content",
        icon: "menu_book",
        endpoints: &[
            ApiEndpoint { method: "GET", path: "/api/v1/courses", desc: "List visible courses", auth: "Bearer" },
            ApiEndpoint { method: "GET", path: "/api/v1/courses/{id}", desc: "Course detail with modules", auth: "Bearer" },
            ApiEndpoint { method: "POST", path: "/api/v1/courses", desc: "Create a course (lecturer)", auth: "Bearer · lecturer" },
            ApiEndpoint { method: "PUT", path: "/api/v1/courses/{id}", desc: "Update course metadata", auth: "Bearer · owner" },
        ],
    },
    ApiGroup {
        name: "Assignments & Quizzes",
        icon: "assignment",
        endpoints: &[
            ApiEndpoint { method: "GET", path: "/api/v1/assignments", desc: "List assignments for my courses", auth: "Bearer" },
            ApiEndpoint { method: "POST", path: "/api/v1/assignments/{id}/submissions", desc: "Submit work for grading", auth: "Bearer · student" },
            ApiEndpoint { method: "POST", path: "/api/v1/quizzes/{id}/attempts", desc: "Start a graded quiz attempt", auth: "Bearer · student" },
            ApiEndpoint { method: "GET", path: "/api/v1/gradebook", desc: "Grades per course (lecturer)", auth: "Bearer · lecturer" },
        ],
    },
    ApiGroup {
        name: "Plugins & Registry",
        icon: "extension",
        endpoints: &[
            ApiEndpoint { method: "GET", path: "/api/v1/plugins", desc: "Registered plugins & status", auth: "Bearer · dev" },
            ApiEndpoint { method: "POST", path: "/api/v1/plugins", desc: "Register a new plugin build", auth: "Key · dev" },
            ApiEndpoint { method: "POST", path: "/api/v1/plugins/{id}/verify", desc: "Verify signature before install", auth: "Key · admin" },
            ApiEndpoint { method: "DELETE", path: "/api/v1/plugins/{id}", desc: "Remove a plugin from registry", auth: "Key · admin" },
        ],
    },
];

// --- Webhooks ---------------------------------------------------------------

pub(crate) struct Webhook {
    pub name: &'static str,
    pub url: &'static str,
    pub events: &'static str,
    pub status: &'static str,
}

pub(crate) const WEBHOOKS: &[Webhook] = &[
    Webhook { name: "grade-publisher", url: "https://hooks.lupane.ac.zw/grade-pub", events: "submission.graded", status: "healthy" },
    Webhook { name: "course-sync", url: "https://lms.bit.ac.zw/hooks/elupane", events: "course.created, course.updated", status: "healthy" },
    Webhook { name: "plugin-alerts", url: "https://ops.elupane.dev/alert", events: "plugin.stopped, plugin.degraded", status: "degraded" },
    Webhook { name: "audit-export", url: "https://storage.elupane.dev/audit", events: "user.role_changed, user.suspended", status: "healthy" },
];

// --- Permissions (RBAC) -----------------------------------------------------

pub(crate) const ROLES: &[&str] = &["Super admin", "Institution admin", "Lecturer", "Student", "Developer"];
pub(crate) const ROLE_ICONS: &[(&str, &str)] = &[
    ("Super admin", "admin_panel_settings"),
    ("Institution admin", "account_balance"),
    ("Lecturer", "co_present"),
    ("Student", "school"),
    ("Developer", "code_blocks"),
];

pub(crate) const PERMISSIONS: &[(&str, &str)] = &[
    ("Manage users", "group"),
    ("Manage courses", "menu_book"),
    ("Grade submissions", "fact_check"),
    ("Approve plugins", "verified"),
    ("Issue signing keys", "key"),
    ("View analytics", "monitoring"),
    ("Send announcements", "campaign"),
    ("Manage institutions", "domain"),
];

/// Initial permission matrix: rows = permissions, cols = roles.
pub(crate) const PERM_MATRIX: &[&[bool]] = &[
    &[true, true, false, false, false],
    &[true, true, true, false, false],
    &[false, false, true, false, false],
    &[true, false, false, false, true],
    &[true, false, false, false, true],
    &[true, true, true, false, true],
    &[true, true, false, false, false],
    &[true, true, false, false, false],
];

// --- Audit log --------------------------------------------------------------

pub(crate) const AUDIT_LOG: &[(&str, &str, &str, &str)] = &[
    ("key", "Signing key issued to developer M. Ncube", "Platform Admin · 4h ago", "amber"),
    ("verified", "Plugin `analytics v0.3.0` signed & approved", "Platform Admin · 4h ago", "teal"),
    ("group_add", "214 students bulk-imported (Lupane State)", "Institution Admin · 1d ago", "indigo"),
    ("domain", "Harare School of Computing onboarded", "Platform Admin · 2d ago", "green"),
    ("person_off", "User `l.sibanda@hsc.ac.zw` suspended", "Institution Admin · 2d ago", "purple"),
    ("shield", "Role `Institution admin` created for BIT", "Platform Admin · 3d ago", "teal"),
];

// --- Notifications ----------------------------------------------------------

pub(crate) const NOTIFICATIONS: &[(&str, &str, &str, &str)] = &[
    ("rate_review", "Dr. Dlamini graded your CS101 assignment", "87% · 12 min ago", "teal"),
    ("quiz", "Week 4 quiz · Functions opens Thursday 14:00", "CS101 · 1h ago", "indigo"),
    ("campaign", "Examination timetable published", "Registrar · 2h ago", "amber"),
    ("event", "WEB302 lab moved to Lab 5 this afternoon", "2h ago", "green"),
    ("bolt", "Your streak is at 12 days — keep it going!", "3h ago", "purple"),
];

// --- Lessons (live classroom) -----------------------------------------------

pub(crate) struct Lesson {
    pub id: &'static str,
    pub title: &'static str,
    pub course: &'static str,
    pub date: &'static str,
    pub time: &'static str,
    pub duration: &'static str,
    pub status: &'static str, // live | scheduled | ended
    pub host: &'static str,
    pub participants: u32,
    pub capacity: u32,
}

pub(crate) const LESSONS: &[Lesson] = &[
    Lesson { id: "lsn-001", title: "Divide & conquer: sorting algorithms deep dive", course: "CS101 · Data Structures", date: "Today", time: "09:00 – 10:30", duration: "90 min", status: "live", host: "Dr. Nkosana Dlamini", participants: 34, capacity: 40 },
    Lesson { id: "lsn-002", title: "SQL joins & query optimisation", course: "DB201 · Databases", date: "Thu, 4 Sep", time: "11:00 – 12:00", duration: "60 min", status: "scheduled", host: "Dr. Nkosana Dlamini", participants: 0, capacity: 40 },
    Lesson { id: "lsn-003", title: "Dioxus state management workshop", course: "WEB302 · Web Development", date: "Fri, 5 Sep", time: "14:00 – 16:00", duration: "120 min", status: "scheduled", host: "Dr. Nkosana Dlamini", participants: 0, capacity: 30 },
    Lesson { id: "lsn-004", title: "Revision: functions & recursion", course: "MATH150 · Calculus", date: "Tue, 26 Aug", time: "10:00 – 11:00", duration: "60 min", status: "ended", host: "Prof. L. Sibanda", participants: 38, capacity: 40 },
];

/// Mock classmates in the live room: (name, initials, tone, mic_on, speaking).
pub(crate) const ROOM_PARTICIPANTS: &[(&str, &str, &str, bool, bool)] = &[
    ("Aisha Patel", "AP", "teal", true, true),
    ("Sipho Ndlovu", "SN", "indigo", true, false),
    ("Naledi Mokoena", "NM", "green", false, false),
    ("Tendai Moyo", "TM", "amber", true, false),
    ("Kgotso Molefe", "KM", "purple", false, false),
    ("Lerato Dube", "LD", "teal", false, false),
];

// Browser-media JavaScript (runs via `document::eval` on the web client only;
// on the server `document` is a no-op, so this is SSR-safe).
pub(crate) const LOCAL_MEDIA_JS: &str = r#"(async () => {
  try {
    const stream = await navigator.mediaDevices.getUserMedia({ video: { width: 1280 }, audio: true });
    const v = document.getElementById('elupane-local-video');
    if (v) { v.srcObject = stream; }
    window.__elupaneLocalStream = stream;
  } catch (e) {}
})();"#;

pub(crate) const START_SHARE_JS: &str = r#"(async () => {
  try {
    const stream = await navigator.mediaDevices.getDisplayMedia({ video: true });
    const attach = () => {
      const v = document.getElementById('elupane-share-video');
      if (v) { v.srcObject = stream; return true; }
      return false;
    };
    if (!attach()) { setTimeout(attach, 120); }
    stream.getVideoTracks()[0] && stream.getVideoTracks()[0].addEventListener('ended', () => {
      const v = document.getElementById('elupane-share-video');
      if (v) { v.srcObject = null; }
    });
    window.__elupaneShareStream = stream;
  } catch (e) {}
})();"#;

pub(crate) const STOP_SHARE_JS: &str = r#"(function() {
  const s = window.__elupaneShareStream;
  if (s) { s.getTracks().forEach(function (t) { t.stop(); }); }
  const v = document.getElementById('elupane-share-video');
  if (v) { v.srcObject = null; }
})();"#;

pub(crate) const STOP_MEDIA_JS: &str = r#"(function() {
  ['__elupaneLocalStream', '__elupaneShareStream'].forEach(function (k) {
    const s = window[k];
    if (s) { s.getTracks().forEach(function (t) { t.stop(); }); }
  });
})();"#;


