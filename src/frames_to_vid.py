import cv2
import os
import sys

image_folder = 'frames_tmp'
vid_name = sys.argv[1]

images = [img for img in os.listdir(image_folder) if img.endswith(".png")]
frame = cv2.imread(os.path.join(image_folder, images[0]))
height, width, layers = frame.shape

# Create VideoWriter with 30fps playback speed
video = cv2.VideoWriter(vid_name, 0, 30, (width, height))

for image in images:
    video.write(cv2.imread(os.path.join(image_folder, image)))

# Clean up
cv2.destroyAllWindows()
video.release()

print('Export complete')
