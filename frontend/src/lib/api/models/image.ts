export enum ImageFit {
  Contain = "Contain",
  Cover = "Cover",
  Width = "Width",
  Height = "Height"
}

export const imageFitText: Record<ImageFit, string> = {
  [ImageFit.Contain]: "Fit the entire image",
  [ImageFit.Cover]: "Fill the available space",
  [ImageFit.Width]: "Fill available width",
  [ImageFit.Height]: "Fill available height"
};

// UUID to an image on the server
export type ImageRef = string;
