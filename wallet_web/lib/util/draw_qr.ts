import { qrcodegen } from "@/util/qrcodegen"

// Draws the given QR Code and sets the canvas size to equel the QR size. The drawn image is purely
// black and white. Can leave a transparent square in the middle to leave space for a logo. The
// percentage must be a number between 0 and 100.
export function drawCanvas(qr: qrcodegen.QrCode, canvas: HTMLCanvasElement, perc: number): void {
  canvas.width = qr.size
  canvas.height = qr.size

  // leave a percentage space in order to show a background
  const mid = Math.floor(qr.size / 2)
  const margin = Math.floor(qr.size * (perc / 2 / 100)) // take a margin
  const even = (qr.size + 1) % 2

  const ctx = canvas.getContext("2d") as CanvasRenderingContext2D
  for (let y = 0; y < qr.size; y++) {
    for (let x = 0; x < qr.size; x++) {
      if (
        x > mid - margin &&
        x < mid + margin + even &&
        y > mid - margin &&
        y < mid + margin + even
      ) {
        ctx.fillStyle = "rgba(255, 255, 255, 0)"
      } else {
        ctx.fillStyle = qr.getModule(x, y) ? "#000" : "#fff"
      }
      ctx.fillRect(x, y, 1, 1)
    }
  }
}
