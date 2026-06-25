// Server error types
export const enum ServerError {
  MalformedMessage = "MalformedMessage",
  InvalidToken = "InvalidToken",
  InvalidNameLength = "InvalidNameLength",
  UsernameTaken = "UsernameTaken",
  InappropriateName = "InappropriateName",
  NotJoinable = "NotJoinable",
  CapacityReached = "CapacityReached",
  UnknownPlayer = "UnknownPlayer",
  Unexpected = "Unexpected",
  InvalidPermission = "InvalidPermission",
  UnexpectedMessage = "UnexpectedMessage",
  InvalidAnswer = "InvalidAnswer"
}

// Messages for different server errors
export const errorText: Record<ServerError, string> = {
  [ServerError.MalformedMessage]: "Unknown client sent invalid message",
  [ServerError.InvalidToken]:
    "Invalid game code provided, check that you have entered the game code correctly",
  [ServerError.InvalidNameLength]: "Invalid name length",
  [ServerError.UsernameTaken]: "Username already in use",
  [ServerError.InappropriateName]:
    "That name is not allowed/inappropriate choose another name",
  [ServerError.NotJoinable]: "Quiz is not joinable",
  [ServerError.CapacityReached]: "Quiz is full",
  [ServerError.UnknownPlayer]: "Target player not found",
  [ServerError.Unexpected]: "Unexpected error occurred",
  [ServerError.InvalidPermission]: "You don't have permission to do that",
  [ServerError.UnexpectedMessage]: "Client and server out of sync",
  [ServerError.InvalidAnswer]: "Invalid answer type"
};
