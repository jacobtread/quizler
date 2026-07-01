const ms = {
  seconds: (value: number) => value * 1000,
  minutes: (value: number) => ms.seconds(value * 60)
};

export const TOKEN_LENGTH: number = 5;

export const MIN_PLAYER_NAME_LENGTH: number = 1;
export const MAX_PLAYER_NAME_LENGTH: number = 30;

export const MAX_TITLE_LENGTH: number = 70;
export const MAX_DESCRIPTION_LENGTH: number = 150;

export const MAX_ANSWER_LENGTH: number = 150;

export const MIN_ANSWERS: number = 1;
export const MAX_ANSWERS: number = 8;

export const MAX_QUESTIONS: number = 50;

export const MAX_QUESTION_LENGTH = 400;

export const MIN_ANSWER_TIME: number = ms.seconds(1);
export const MAX_ANSWER_TIME: number = ms.minutes(30);

export const MIN_SCORE: number = 0;
export const MAX_MAX_SCORE: number = 10_000;
export const MAX_BONUS_SCORE: number = 10_000;

export const MIN_BONUS_TIME = ms.seconds(1);
export const MAX_BONUS_TIME = ms.minutes(30);

export const MIN_MAX_PLAYERS: number = 1; /* 1 max player min */
export const MAX_MAX_PLAYERS: number = 50; /* 50 max player limit to games */

export const MAX_IMAGE_BYTES: number = 1024 * 1024 * 15; /* 15mb */
