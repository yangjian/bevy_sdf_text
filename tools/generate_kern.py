import sys
import fontforge

font = fontforge.open(sys.argv[1])
font.generate(sys.argv[2], flags=('apple', 'opentype', 'old-kern'))