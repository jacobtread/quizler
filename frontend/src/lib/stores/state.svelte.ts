import type { GameConfig, GameToken, SessionId } from "$api/models";

export const enum AppStateType {
  /// Home screen
  Home = 0,
  // Join Game Screen
  Connect,
  // Create game screen
  Create,
  // App is in a game
  Game
}

export interface GameData {
  // ID of the current player
  id: SessionId;
  // Current game token
  token: GameToken;
  // Current game config
  config: GameConfig;
  // Whether we are the host
  host: boolean;
  // The current player name
  name?: string | undefined;
}

export type AppState =
  | { ty: AppStateType.Home }
  | { ty: AppStateType.Connect }
  | { ty: AppStateType.Create }
  | { ty: AppStateType.Game; gameData: GameData };

export interface AppStateStore {
  current: AppState;
  setHome: VoidFunction;
  setCreate: VoidFunction;
  setConnect: VoidFunction;
  setGame: (gameData: GameData) => void;
}

export function createAppState(): AppStateStore {
  let currentState = $state.raw<AppState>({ ty: AppStateType.Home });

  return {
    get current() {
      return currentState;
    },

    setHome() {
      currentState = { ty: AppStateType.Home };
    },

    setCreate() {
      currentState = { ty: AppStateType.Create };
    },

    setConnect() {
      currentState = { ty: AppStateType.Connect };
    },

    setGame(gameData: GameData) {
      currentState = { ty: AppStateType.Game, gameData };
    }
  };
}
