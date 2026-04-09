from pynput.keyboard import Key, Controller
from time import sleep

KB = Controller()
PAUSE = 0.5

def goto(link):
    KB.press(Key.cmd)
    KB.release(Key.cmd)

    sleep(PAUSE)

    KB.type("Chrome")

    sleep(PAUSE)

    KB.press(Key.enter)
    KB.release(Key.enter)

    sleep(PAUSE)

    KB.type(link)

    sleep(PAUSE)

    KB.press(Key.enter)
    KB.release(Key.enter)

if __name__ == '__main__':
    goto("criticalanglestudios.com")