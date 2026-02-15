"""
生成 WeRun 应用程序图标
"""
from PIL import Image, ImageDraw

def create_icon():
    img_sizes = [(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    images = []
    
    for size in img_sizes:
        img = Image.new('RGBA', size, (0, 0, 0, 0))
        draw = ImageDraw.Draw(img)
        
        center = size[0] // 2
        radius = int(size[0] * 0.4)
        
        draw.ellipse(
            [center - radius, center - radius, center + radius, center + radius],
            fill=(66, 133, 244, 255),
            outline=(33, 66, 122, 255),
            width=max(1, size[0] // 32)
        )
        
        arrow_width = max(2, size[0] // 8)
        arrow_height = int(radius * 0.6)
        
        draw.polygon([
            (center - arrow_width, center - arrow_height // 2),
            (center + arrow_width, center - arrow_height // 2),
            (center, center + arrow_height // 2)
        ], fill=(255, 255, 255, 255))
        
        images.append(img)
    
    images[0].save(
        'icon.ico',
        format='ICO',
        sizes=img_sizes,
        append_images=images[1:]
    )
    print("图标 icon.ico 已生成!")

if __name__ == '__main__':
    create_icon()
