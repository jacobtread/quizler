// Reading, writing and validation logic for the quizler file format

import {
  imageStore,
  loadImagePreview,
  type StoredImage
} from "$stores/imageStore";
import {
  type Question,
  type CreateData,
  type CreateDataRuntime
} from "$api/models";
import { z } from "zod";
import { v4 } from "uuid";
import { createDataSchema } from "$lib/api/schema/game";

const fileFormatSchema = createDataSchema.and(
  z.object({
    images: z.array(
      z.object({
        uuid: z.string().uuid(),
        name: z.string(),
        size: z.number(),
        type: z.string(),
        data: z.array(z.number())
      })
    )
  })
);

type QuizFormat = z.infer<typeof fileFormatSchema>;
type SerializedImage = QuizFormat["images"][0];

/**
 * Converts the provided stored image into a JSON
 * serializable version by converting the stored
 * blob into a array of bytes
 *
 * @param image The image to convert
 * @returns The converted image
 */
async function serializeImage(image: StoredImage): Promise<SerializedImage> {
  const buffer = await image.blob.arrayBuffer();
  return {
    uuid: image.uuid,
    name: image.name,
    size: image.size,
    type: image.blob.type,
    data: Array.from(new Uint8Array(buffer)) as number[]
  };
}

/**
 * Serializes the provided quiz contents into a blob
 *
 * @param data The quiz data
 * @param storeImages The stored images
 * @returns Promise to the encoded quiz blob
 */
export async function createQuizBlob(
  data: CreateData,
  storeImages: StoredImage[]
): Promise<Blob> {
  // Convert the stored images into a serializable form
  const images: SerializedImage[] = await Promise.all(
    storeImages.map(serializeImage)
  );

  // Create the object to serialize as the quiz file
  const output: QuizFormat = {
    ...data,
    images
  };

  const json = JSON.stringify(output);

  // Blob created from the serialized JSON
  return new Blob([json], { type: "application/json" });
}

/**
 * Loads a quiz from the provided file blob
 *
 * @param file The file to load the quiz from
 * @throws {ZodError} If the validation failed
 * @returns The loaded quiz data
 */
export async function loadQuizBlob(file: Blob): Promise<CreateDataRuntime> {
  const reader = new FileReader();

  // Await the reading process
  await new Promise((resolve, reject) => {
    reader.onload = resolve;
    reader.onerror = reject;
    reader.onabort = reject;

    // Load the file as a string
    reader.readAsText(file);
  });

  const data = reader.result as string;

  const parsed = JSON.parse(data);

  const obj: QuizFormat = fileFormatSchema.parse(parsed);

  for (const { data, uuid, name, type, size } of obj.images) {
    // Convert the input data array into a blob
    const dataArray = new Uint8Array(data);
    const blob = new Blob([dataArray.buffer], { type });

    // Add the image to the image store
    imageStore.update((store) => {
      // Remove duplicates from loading the file again
      const filtered = store.filter((value) => value.uuid !== uuid);

      // Add the new item
      const item = { uuid, name, size, blob };
      return [...filtered, item];
    });

    // Trigger the image preview loading
    loadImagePreview(blob, uuid);
  }

  // Assign IDs to each of the question elements
  obj.questions.forEach((value) => {
    (value as Question).id = v4();
  });

  return {
    name: obj.name,
    text: obj.text,
    max_players: obj.max_players,
    filtering: obj.filtering,
    questions: obj.questions as Question[]
  };
}
