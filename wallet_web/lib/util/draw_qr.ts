import { qrcodegen } from "@/util/qrcodegen"

// Draws the given QR Code and sets the canvas size to equel the QR size. The drawn image is purely
// black and white.
export function drawCanvas(qr: qrcodegen.QrCode, canvas: HTMLCanvasElement): void {
  canvas.width = qr.size
  canvas.height = qr.size

  const ctx = canvas.getContext("2d") as CanvasRenderingContext2D
  for (let y = 0; y < qr.size; y++) {
    for (let x = 0; x < qr.size; x++) {
      ctx.fillStyle = qr.getModule(x, y) ? "#000" : "#fff"

      ctx.fillRect(x, y, 1, 1)
    }
  }
}
