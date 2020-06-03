import os
import sys

try:
    from natsort import natsorted
    import cv2
except ImportError:
    print('1', end='')
    raise ImportError('Unmet dependencies')

# Make sure script is running in python3
python_version = sys.version_info[0]
if python_version != 3:
    print('2', end='')
    raise Exception('Script must run with python3')

vid_name = sys.argv[1]
image_folder = sys.argv[2]

images = [img for img in os.listdir(image_folder) if img.endswith(".png")]
frame = cv2.imread(os.path.join(image_folder, images[0]))
height, width, layers = frame.shape

# Create VideoWriter with 30fps playback speed
video = cv2.VideoWriter(vid_name, 0, 30, (width, height))

# Use natsorted to make sure that frames are added to the video in order
for image in natsorted(images, key=lambda y: y.lower()):
    video.write(cv2.imread(os.path.join(image_folder, image)))

# Clean up
cv2.destroyAllWindows()
video.release()

print('0', end='')
