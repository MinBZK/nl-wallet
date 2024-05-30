import { type EngagementOptions, type EngagementResponse } from "@/models/engagement"
import axios from "axios"

export const createEngagement = async (
  baseURL: string,
  options: EngagementOptions,
): Promise<EngagementResponse> => {
  const response = await axios.post(`${baseURL}/engagement`, options)
  return await response.data
}
