package helper

import org.openqa.selenium.OutputType
import org.openqa.selenium.WebElement
import java.awt.Color
import java.awt.Image
import java.awt.RenderingHints
import java.awt.image.BufferedImage
import java.io.ByteArrayInputStream
import javax.imageio.ImageIO
import kotlin.math.abs
import kotlin.math.sqrt

/**
 * Helper class for detecting whether a UI element contains a portrait photo.
 * This is performed based on simple image analysis (contrast and color distribution).
 */
class PortraitImageHelper {

    /**
     * Checks if the screenshot of the specified [element] resembles a portrait photo.
     *
     * @param element The WebElement to take a screenshot of.
     * @param size The width and height to scale the image to for analysis.
     * @return True if the image meets the criteria for a photo, false otherwise.
     */
    fun hasPortraitImage(element: WebElement, size: Int = 200): Boolean {
        val bytes = element.getScreenshotAs(OutputType.BYTES)
        val img = ImageIO.read(ByteArrayInputStream(bytes)) ?: return false
        // Scale the image to a smaller grayscale format for efficient analysis
        return imageLooksLikePhoto(resizeToGray(img, size, size))
    }

    /**
     * Analyzes grayscale statistics of the image to determine if it is a photo.
     * It evaluates the mean, standard deviation (contrast), and the number of pixels deviating from neutral gray.
     */
    private fun imageLooksLikePhoto(img: BufferedImage): Boolean {
        var mean = 0.0
        var variance = 0.0
        var contentPixels = 0

        // Calculate mean and count pixels with 'content'
        for (y in 0 until img.height) {
            for (x in 0 until img.width) {
                val gray = Color(img.getRGB(x, y), true).red
                mean += gray
                // Count pixels that significantly deviate from the middle (128), indicating detail
                if (abs(gray - 128.0) > 20.0) contentPixels++
            }
        }

        mean /= (img.width * img.height).toDouble()

        // Calculate variance for standard deviation (measure of contrast)
        for (y in 0 until img.height) {
            for (x in 0 until img.width) {
                val gray = Color(img.getRGB(x, y), true).red
                val delta = gray - mean
                variance += delta * delta
            }
        }

        variance /= (img.width * img.height).toDouble()
        val stdDev = sqrt(variance)

        // Heuristic criteria:
        // - mean: Neither too dark nor too bright.
        // - stdDev: Sufficient contrast present (not a flat surface).
        // - contentPixels: Enough pixels with deviating values (detail).
        return mean > 20.0 && mean < 220.0 && stdDev > 18.0 && contentPixels > 1500
    }

    /**
     * Scales an image and converts it to grayscale (8-bit).
     */
    private fun resizeToGray(src: BufferedImage, targetWidth: Int, targetHeight: Int): BufferedImage {
        val out = BufferedImage(targetWidth, targetHeight, BufferedImage.TYPE_BYTE_GRAY)
        val g2 = out.createGraphics()
        g2.setRenderingHint(RenderingHints.KEY_INTERPOLATION, RenderingHints.VALUE_INTERPOLATION_BILINEAR)
        g2.drawImage(src.getScaledInstance(targetWidth, targetHeight, Image.SCALE_SMOOTH), 0, 0, null)
        g2.dispose()
        return out
    }
}
