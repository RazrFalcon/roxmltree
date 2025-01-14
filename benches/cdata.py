import string
import random

letters = [letter for letter in string.ascii_letters]
lines = ["\r\n", "\n"]
chunks = letters + lines

with open("cdata.xml", "w") as file:
    file.write("<root><![CDATA[")

    for _ in range(100_000):
        file.write(random.choice(chunks))

    file.write("]]></root>")
