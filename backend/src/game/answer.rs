use crate::types::{Answer, AnswerData, AnswerValue, ImStr, Question, QuestionData, Score};

/// Structure storing the player answers. Fixed length to
/// the total number of questions in the game
pub struct PlayerAnswers {
    /// The actual player answers
    values: Box<[PlayerAnswer]>,
}

impl PlayerAnswers {
    /// Creates a new player answers structure of the
    /// provided length
    ///
    /// # Arguments
    /// * length - The length of the answers
    pub fn new(length: usize) -> Self {
        // Create all the answers collecting into the boxed slice
        let values: Box<[PlayerAnswer]> = (0..length).map(|_| PlayerAnswer::default()).collect();
        Self { values }
    }

    /// Resets the state of each player answer replacing the
    /// score and answer data with None
    pub fn reset(&mut self) {
        self.values.iter_mut().for_each(|value| {
            value.data = None;
            value.score = None;
        })
    }

    /// Sets the player answer at the provided index to the
    /// provided value
    ///
    /// # Arguments
    /// * index - The index of the answer within the values array
    /// * answer - The answer to set the value to
    pub fn set_answer(&mut self, index: usize, answer: AnswerData) {
        debug_assert!(index < self.values.len());
        self.values[index].data = Some(answer);
    }

    /// Provides mutable access to the player answer at the provided
    /// index
    ///
    /// # Arguments
    /// * index - The index of the answer within the values array
    pub fn get_answer(&mut self, index: usize) -> &mut PlayerAnswer {
        debug_assert!(index < self.values.len());
        &mut self.values[index]
    }

    /// Provides a borrow to the player answer at the provided
    /// index
    ///
    /// # Arguments
    /// * index - The index of the answer within the values array
    pub fn get_answer_ref(&self, index: usize) -> &PlayerAnswer {
        debug_assert!(index < self.values.len());
        &self.values[index]
    }

    /// Checks if theres an answer stored at the provided index
    ///
    /// # Arguments
    /// * index - The index of the answer within the values array
    pub fn has_answer(&self, index: usize) -> bool {
        debug_assert!(index < self.values.len());
        self.values[index].data.is_some()
    }
}

/// Structure storing a player answer and the score provided
/// for it
#[derive(Default)]
pub struct PlayerAnswer {
    /// The answer provided by the player
    data: Option<AnswerData>,
    /// The score provided by the server
    score: Option<Score>,
}

impl PlayerAnswer {
    /// Marks the current question updating the stored score
    /// and returns the score
    ///
    /// # Arguments
    /// * question - The question to mark this answer against
    pub fn mark(&mut self, question: &Question) -> Score {
        let score = self.mark_impl(question);
        self.score = Some(score);
        score
    }

    /// Get the current score value if the answer has been scored
    pub fn score(&self) -> Option<&Score> {
        self.score.as_ref()
    }

    /// Marking implementation which marks the current answer
    /// using the provided question as the correct answers.
    ///
    /// # Arguments
    /// * question - The question to mark this answer against
    fn mark_impl(&self, question: &Question) -> Score {
        let answer = match &self.data {
            Some(value) => value,
            None => return Score::Incorrect,
        };

        let elapsed_ms = answer.elapsed.as_millis() as u32;
        let is_bonus = elapsed_ms <= question.bonus_score_time;

        // Calculate the % amount between the min and max answer times
        let answer_time_percent = 1.0 - ((elapsed_ms as f32) / (question.answer_time as f32));

        let scoring = &question.scoring;

        // The base score from the answer time and the bonus
        let mut base_score = scoring.min_score
            + ((scoring.max_score - scoring.min_score) as f32 * answer_time_percent) as u32;

        // Append bonus score amount
        if is_bonus {
            base_score += scoring.bonus_score;
        }

        use Answer as A;
        use QuestionData as Q;

        match (&answer.answer, &question.data) {
            (A::Single { answer }, Q::Single { answers, .. }) => {
                Self::mark_single(*answer, answers, base_score)
            }
            (A::Multiple { answers: indexes }, Q::Multiple { answers, .. }) => {
                Self::mark_multiple(indexes, answers, base_score)
            }
            (A::TrueFalse { answer }, Q::TrueFalse { answer: actual }) => {
                Self::mark_bool(*answer, *actual, base_score)
            }
            (
                A::Typer { answer },
                Q::Typer {
                    answers,
                    ignore_case,
                },
            ) => Self::mark_typer(answer, answers, *ignore_case, base_score),
            // Mismatched types shouldn't be possible but
            // will be marked as incorrect
            _ => Score::Incorrect,
        }
    }

    /// Marks a single choice question
    ///
    /// # Arguments
    /// * answer - The index of the users answer
    /// * answers - The answers for the question
    /// * base_score - The base score for correct answers
    fn mark_single(answer: usize, answers: &[AnswerValue], base_score: u32) -> Score {
        let is_valid = answers
            .get(answer)
            .map(|value| value.correct)
            .unwrap_or(false);
        if is_valid {
            Score::Correct { value: base_score }
        } else {
            Score::Incorrect
        }
    }

    /// Marks a multiple choice question
    ///
    /// # Arguments
    /// * indexes - The indexes of the answers the player chose
    /// * answers - The answers for the question
    /// * base_score - The base score for correct answers
    fn mark_multiple(indexes: &[usize], answers: &[AnswerValue], base_score: u32) -> Score {
        let count_answers = indexes.len();

        // The total number of actual correct answers
        let count_expected = answers.iter().filter(|value| value.correct).count();

        // Didn't provide enough answer or provided too many
        if count_answers < 1 || count_answers > count_expected {
            return Score::Incorrect;
        }

        // Count the number of provided correct answers
        let count_correct = indexes
            .iter()
            .filter_map(|index| answers.get(*index))
            .filter(|value| value.correct)
            .count();

        if count_correct < 1 {
            Score::Incorrect
        } else if count_correct == count_expected {
            Score::Correct { value: base_score }
        } else {
            // % correct out of total answers
            let percent = count_correct as f32 / count_expected as f32;
            let score = ((base_score as f32) * percent).round() as u32;
            Score::Partial {
                value: score,
                count: count_correct as u32,
                total: count_expected as u32,
            }
        }
    }

    /// Marks a True / False boolean question
    ///
    /// # Arguments
    /// * answer - The boolean answer the player chose
    /// * actual - The correct answer for the question
    /// * base_score - The base score for correct answers
    fn mark_bool(answer: bool, actual: bool, base_score: u32) -> Score {
        if answer == actual {
            Score::Correct { value: base_score }
        } else {
            Score::Incorrect
        }
    }

    /// Marks a typing question
    ///
    /// # Arguments
    /// * answer - The player typed answer
    /// * answers - The question valid answers
    /// * ignore_case - Whether to ignore case when matching
    /// * base_score - The base score for correct answers
    fn mark_typer(answer: &str, answers: &[ImStr], ignore_case: bool, base_score: u32) -> Score {
        // Trim extra whitespace
        let answer = answer.trim();

        let equal_fn = if ignore_case {
            str::eq_ignore_ascii_case
        } else {
            str::eq
        };

        let correct = answers.iter().any(|value| equal_fn(answer, value));

        if correct {
            Score::Correct { value: base_score }
        } else {
            Score::Incorrect
        }
    }
}

#[cfg(test)]
mod test {
    use super::PlayerAnswer;
    use crate::types::{AnswerValue, Score};

    pub const TEST_CORRECT_SCORE: u32 = 100;

    #[test]
    fn test_mark_single_correct() {
        let answers = &[
            AnswerValue {
                value: "Test".into(),
                correct: true,
            },
            AnswerValue {
                value: "Test".into(),
                correct: false,
            },
        ];

        let score = PlayerAnswer::mark_single(0, answers, TEST_CORRECT_SCORE);
        let expected = Score::Correct {
            value: TEST_CORRECT_SCORE,
        };
        assert_eq!(score, expected);
    }

    #[test]
    fn test_mark_single_incorrect() {
        let answers = &[
            AnswerValue {
                value: "Test".into(),
                correct: true,
            },
            AnswerValue {
                value: "Test".into(),
                correct: false,
            },
        ];

        let score = PlayerAnswer::mark_single(1, answers, TEST_CORRECT_SCORE);
        let expected = Score::Incorrect;

        assert_eq!(score, expected);
    }

    #[test]
    fn test_mark_multiple_correct() {
        let answers = &[
            AnswerValue {
                value: "Test".into(),
                correct: true,
            },
            AnswerValue {
                value: "Test".into(),
                correct: false,
            },
            AnswerValue {
                value: "Test".into(),
                correct: true,
            },
            AnswerValue {
                value: "Test".into(),
                correct: false,
            },
        ];

        let score = PlayerAnswer::mark_multiple(&[0, 2], answers, TEST_CORRECT_SCORE);
        let expected = Score::Correct {
            value: TEST_CORRECT_SCORE,
        };

        assert_eq!(score, expected);
    }

    #[test]
    fn test_mark_multiple_partial() {
        let answers = &[
            AnswerValue {
                value: "Test".into(),
                correct: true,
            },
            AnswerValue {
                value: "Test".into(),
                correct: false,
            },
            AnswerValue {
                value: "Test".into(),
                correct: true,
            },
            AnswerValue {
                value: "Test".into(),
                correct: false,
            },
        ];

        let score = PlayerAnswer::mark_multiple(&[1, 2], answers, TEST_CORRECT_SCORE);
        let expected = Score::Partial {
            value: TEST_CORRECT_SCORE / 2,
            count: 1,
            total: 2,
        };

        assert_eq!(score, expected);
    }

    #[test]
    fn test_mark_multiple_incorrect() {
        let answers = &[
            AnswerValue {
                value: "Test".into(),
                correct: true,
            },
            AnswerValue {
                value: "Test".into(),
                correct: false,
            },
            AnswerValue {
                value: "Test".into(),
                correct: true,
            },
            AnswerValue {
                value: "Test".into(),
                correct: false,
            },
        ];

        let score = PlayerAnswer::mark_multiple(&[1, 3], answers, TEST_CORRECT_SCORE);
        let expected = Score::Incorrect;

        assert_eq!(score, expected);
    }

    #[test]
    fn test_mark_bool_correct() {
        let score = PlayerAnswer::mark_bool(true, true, TEST_CORRECT_SCORE);
        let expected = Score::Correct {
            value: TEST_CORRECT_SCORE,
        };

        assert_eq!(score, expected);
    }

    #[test]
    fn test_mark_bool_incorrect() {
        let score = PlayerAnswer::mark_bool(true, false, TEST_CORRECT_SCORE);
        let expected = Score::Incorrect;

        assert_eq!(score, expected);
    }

    #[test]
    fn test_mark_typer_correct_case_sensitive() {
        let score = PlayerAnswer::mark_typer(
            "Test",
            &["Test".into(), "Test 1".into(), "Test 2".into()],
            false,
            TEST_CORRECT_SCORE,
        );
        let expected = Score::Correct {
            value: TEST_CORRECT_SCORE,
        };

        assert_eq!(score, expected);
    }

    #[test]
    fn test_mark_typer_correct_case_insensitive() {
        let score = PlayerAnswer::mark_typer(
            "test",
            &["Test".into(), "Test 1".into(), "Test 2".into()],
            true,
            TEST_CORRECT_SCORE,
        );
        let expected = Score::Correct {
            value: TEST_CORRECT_SCORE,
        };

        assert_eq!(score, expected);
    }

    #[test]
    fn test_mark_typer_incorrect_case_sensitive() {
        let score = PlayerAnswer::mark_typer(
            "test",
            &["Test".into(), "Test 1".into(), "Test 2".into()],
            false,
            TEST_CORRECT_SCORE,
        );
        let expected = Score::Incorrect;

        assert_eq!(score, expected);
    }

    #[test]
    fn test_mark_typer_incorrect_case_insensitive() {
        let score = PlayerAnswer::mark_typer(
            "test 5",
            &["Test".into(), "Test 1".into(), "Test 2".into()],
            true,
            TEST_CORRECT_SCORE,
        );
        let expected = Score::Incorrect;

        assert_eq!(score, expected);
    }
}
