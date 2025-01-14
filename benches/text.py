import string
import random

letters = [letter for letter in string.ascii_letters]
spaces = ["\r\n", "\n"]
codes = ["&quot;", "&amp;", "&apos;", "&lt;", "&gt;", "&#x09;", "&#x0D;", "&#x0A;"]
refs = [f"&entity{idx};" for idx in range(5)]
chunks = 5 * letters + spaces + codes
chunks_with_refs = chunks + refs

with open("text.xml", "w") as file:
    file.write("<!DOCTYPE test [\n")

    for idx in range(5):
        file.write(f"<!ENTITY entity{idx} '")

        for _ in range(300):
            file.write(random.choice(chunks))

        file.write("'>")

    file.write("]>\n\n")

    file.write("<root>")

    for _ in range(100_000):
        file.write(random.choice(chunks_with_refs))

    file.write("</root>")
