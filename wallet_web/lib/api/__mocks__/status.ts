import { Status } from "@/models/status"
import { vi } from "vitest"

const statusResponse = {
  status: Status.Created,
  engagement_url: "walletdebuginteraction://wallet.edi.rijksoverheid.nl/disclosure/owBjMS4wAYIB2BhYS6QBAiABIVggtMJb_c0CvogL2byTBOyVxsW0UsfOOtmmO-nS3zovWgQiWCCKo-9tc6oOLQC_8ZWpkDVHikgQRNJkQA2jWNGRaj2X4wKBgwQBgXhBaHR0cDovL2xvY2FsaG9zdDozMDAxL2Rpc2Nsb3N1cmUvbWt3TDBzSGZQMmNMSmNSTXVEekNIWEVvZnVqazlubmw?session_type=cross_device"
}

export const getStatus = vi.fn().mockImplementation(async () => statusResponse)
