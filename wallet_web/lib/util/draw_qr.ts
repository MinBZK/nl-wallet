import { qrcodegen } from "@/util/qrcodegen"

// Draws the given QR Code, with the given module scale and border modules, onto the given HTML
// canvas element. The canvas's width and height is resized to (qr.size + border * 2) * scale.
// The drawn image is purely dark and light, and fully opaque.
// The scale must be a positive integer and the border must be a non-negative integer.
export function drawCanvas(qr: qrcodegen.QrCode, scale: number, border: number, lightColor: string, darkColor: string, canvas: HTMLCanvasElement): void {
  if (scale <= 0 || border < 0) {
    throw new RangeError("Value out of range")
  }
  const size: number = (qr.size + border * 2) * scale
  canvas.width = size
  canvas.height = size
  const ctx = canvas.getContext("2d") as CanvasRenderingContext2D
  for (let y = -border; y < qr.size + border; y++) {
    for (let x = -border; x < qr.size + border; x++) {
      ctx.fillStyle = qr.getModule(x, y) ? darkColor : lightColor
      ctx.fillRect((x + border) * scale, (y + border) * scale, scale, scale)
    }
  }
}
