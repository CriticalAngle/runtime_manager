from pynput.mouse import Controller
import random
from time import sleep

mouse = Controller()

for _ in range(15):
    mouse.move(int(random.random() * 1000), int(random.random() * 1000))
    sleep(1)