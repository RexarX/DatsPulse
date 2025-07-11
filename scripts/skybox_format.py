from PIL import Image

# List your images in the correct order
face_files = [
    "right.png",   # +X
    "left.png",    # -X
    "top.png",     # +Y
    "bottom.png",  # -Y
    "front.png",   # +Z
    "back.png",    # -Z
]

# Open all images
faces = [Image.open(f) for f in face_files]

# Check all are the same size
w, h = faces[0].size
assert all(im.size == (w, h) for im in faces), "All faces must be the same size!"

# Create new image (width, height*6)
strip = Image.new("RGBA", (w, h * 6))

# Paste each face in order
for i, face in enumerate(faces):
    strip.paste(face, (0, i * h))

strip.save("cubemap_strip.png")
print("Saved cubemap_strip.png")
