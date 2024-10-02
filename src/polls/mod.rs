

#[derive(Debug, Clone, Default)]
pub struct Poll {
    pub id: String,
    pub title: String,
    pub choices: Vec<PollChoice>,
    pub start_time: i32,
    pub duration: i32
}

#[derive(Debug, Clone, Default)]
pub struct PollChoice {
    pub name: String,
    // pub index: usize
}