// Name filtering modes
export enum NameFiltering {
  // Don't filter names anything goes
  None = "None",
  // Only stop the more severe names
  Low = "Low",
  // Stop anything thats above mild
  Medium = "Medium",
  // Filter out any names that might be inappropriate
  High = "High"
}

export const nameFilterText: Record<NameFiltering, string> = {
  [NameFiltering.None]: "Don't filter names",
  [NameFiltering.Low]: "Filter out more severe names",
  [NameFiltering.Medium]: "Filter out anything thats not mild",
  [NameFiltering.High]: "Filter out as much as possible"
};
