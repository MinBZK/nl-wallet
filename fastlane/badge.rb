require 'fileutils'
require 'pathname'

require 'mini_magick'

class BadgeError < StandardError
end

class Badge
    def initialize(label, font)
        @label = label
        @font = font
        @bg_fill = '#ff9900a0'
        @fg_fill = '#eeeeee'
        @font_rel_size = 0.12
        @scale = 1.2
    end

    def add_to_image(path)
        image = MiniMagick::Image.new(path)
        raise BadgeError.new("ERROR: `#{path}` not a square image") unless image.width == image.height

        font_size = (@font_rel_size * image.height).round
        rect_y0 = ((0.5 + @font_rel_size * 0.6) * image.height).floor
        rect_y1 = ((0.5 - @font_rel_size * 0.6) * image.height).ceil
        center_x = (0.5 * image.width).round
        center_y = (0.5 * image.height).round
        transl_x = (0.74 * image.width).round
        transl_y = (0.74 * image.width).round

        MiniMagick::Utilities.tempfile(path.extname) do |tempfile|
            MiniMagick::Tool::Magick.new do |magick|
                magick << path
                magick.stack do |badge|
                    badge << '-size' << "#{image.width}x#{image.height}" << 'xc:transparent'
                    badge << '-fill' << @bg_fill << '-draw' << "rectangle 0,#{rect_y0} #{image.width},#{rect_y1}"
                    badge << '-font' << @font << '-fill' << @fg_fill << '-pointsize' << font_size
                    badge << '-gravity' << 'center' << '-draw' << "text 0,0 '#{@label}'"
                    badge << '-virtual-pixel' << 'transparent' << '-distort' << 'SRT'
                    badge << "#{center_x},#{center_y} #{@scale} -45 #{transl_x},#{transl_y}"
                end
                magick << '-compose' << 'atop' << '-composite'
                magick << tempfile.path
            end
            FileUtils.mv(tempfile.path, path)
        end
    end

    def add_seasonal_image(path)
        return unless Date.today.month == 12

        image = MiniMagick::Image.new(path)
        add_path = Pathname::new(__FILE__).parent.join('badges').join('tree.png')
        add_image = MiniMagick::Image.new(add_path)

        scale = 0.4 * image.width / add_image.width
        center_x = (0.5 * add_image.width).round
        center_y = (0.5 * add_image.height).round
        transl_x = (0.3 * image.width).round
        transl_y = (0.74 * image.width).round

        MiniMagick::Utilities.tempfile(path.extname) do |tempfile|
            MiniMagick::Tool::Magick.new do |magick|
                magick << path
                magick.stack do |add|
                    add << add_path
                    add << '-virtual-pixel' << 'transparent' << '-distort' << 'SRT'
                    add << "#{center_x},#{center_y} #{scale} 0 #{transl_x},#{transl_y}"
                end
                magick << '-compose' << 'atop' << '-composite'
                magick << tempfile.path
            end
            FileUtils.mv(tempfile.path, path)
        end
    end
end
