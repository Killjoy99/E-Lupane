pub struct CreateCourseRequest {

    pub programme_id: Option<Uuid>,

    pub course_code: String,

    pub course_name: String,

    pub description: Option<String>,

    pub credits: i32,

    pub semester: Option<i32>,

    pub level: Option<i32>,
}