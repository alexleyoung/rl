#!/usr/bin/env python3
"""PyMuPDF sidecar for rl-server PDF extraction.

Usage:
    pdf_extract.py --mode=content <path>
    pdf_extract.py --mode=meta    <path>
    pdf_extract.py --selftest
"""

import sys
import json
import base64
import argparse
import statistics


def selftest():
    import fitz  # noqa: F401
    print("ok")
    sys.exit(0)


def extract_meta(path: str) -> dict:
    import fitz
    doc = fitz.open(path)
    m = doc.metadata or {}
    def clean(v):
        return v.strip() or None if isinstance(v, str) else None
    return {
        "title":       clean(m.get("title")),
        "author":      clean(m.get("author")),
        "description": clean(m.get("subject")),
        "page_count":  doc.page_count,
    }


def _heading_level(size: float, body_size: float, size_levels: list[float]) -> int | None:
    """Map a font size to an h2-h6 level, or None if it's body text."""
    if size <= body_size * 1.05:
        return None
    # size_levels is sorted descending; index 0 → h2, index 1 → h3, etc.
    for i, threshold in enumerate(size_levels):
        if size >= threshold:
            return min(i + 2, 6)
    return None


def extract_content(path: str) -> dict:
    import fitz

    doc = fitz.open(path)

    # Collect all span sizes across the document to find the modal body size.
    all_sizes = []
    page_dicts = []
    for page in doc:
        d = page.get_text("dict", flags=fitz.TEXT_PRESERVE_WHITESPACE)
        page_dicts.append(d)
        for block in d.get("blocks", []):
            if block.get("type") != 0:  # 0 = text
                continue
            for line in block.get("lines", []):
                for span in line.get("spans", []):
                    sz = span.get("size", 0)
                    if sz > 4:
                        all_sizes.append(round(sz, 1))

    body_size = statistics.mode(all_sizes) if all_sizes else 11.0

    # Build size buckets above body_size for h2..h6 (largest first → h2).
    larger = sorted({s for s in all_sizes if s > body_size * 1.05}, reverse=True)
    # Collapse sizes within 0.5pt of each other into one bucket.
    buckets: list[float] = []
    for s in larger:
        if not buckets or buckets[-1] - s > 0.5:
            buckets.append(s)
    # Each bucket maps to h(2 + index), capped at h6.

    pages_out = []
    for page, page_dict in zip(doc, page_dicts):
        blocks_out = []
        raw_blocks = page_dict.get("blocks", [])

        for block in raw_blocks:
            btype = block.get("type")

            if btype == 0:  # text
                # Collect lines; detect heading vs paragraph per line-group.
                # A "block" in PyMuPDF is already a visual paragraph unit.
                lines_text = []
                dominant_size = body_size
                sizes_in_block = []
                for line in block.get("lines", []):
                    line_parts = []
                    for span in line.get("spans", []):
                        t = span.get("text", "")
                        if t.strip():
                            line_parts.append(t)
                            sz = span.get("size", 0)
                            if sz > 4:
                                sizes_in_block.append(sz)
                    joined = "".join(line_parts).strip()
                    if joined:
                        lines_text.append(joined)

                if not lines_text:
                    continue

                text = " ".join(lines_text)
                if sizes_in_block:
                    dominant_size = statistics.median(sizes_in_block)

                level = _heading_level(dominant_size, body_size, buckets)
                # Only treat as heading if it's a short single-line-ish block.
                if level is not None and len(text) <= 200:
                    blocks_out.append({"type": "heading", "level": level, "text": text})
                else:
                    blocks_out.append({"type": "paragraph", "text": text})

            elif btype == 1:  # image
                try:
                    xref = block.get("image")
                    if xref is None:
                        # Older PyMuPDF: image info is in block directly.
                        xref = block.get("xref")
                    if xref is None:
                        continue
                    img_info = doc.extract_image(xref)
                    if not img_info:
                        continue
                    w = img_info.get("width", 0)
                    h = img_info.get("height", 0)
                    if w < 32 or h < 32:
                        continue  # skip tiny decorative glyphs
                    raw = img_info["image"]
                    ext = img_info.get("ext", "png")
                    # Always re-encode as PNG for consistency.
                    pix = fitz.Pixmap(raw)
                    if pix.alpha:
                        pix = fitz.Pixmap(fitz.csRGB, pix)
                    png_bytes = pix.tobytes("png")
                    data_b64 = base64.b64encode(png_bytes).decode("ascii")
                    blocks_out.append({
                        "type": "image",
                        "data": data_b64,
                        "ext": "png",
                        "width": w,
                        "height": h,
                    })
                except Exception as e:
                    print(f"warn: image extraction failed: {e}", file=sys.stderr)
                    continue

        pages_out.append({"blocks": blocks_out})

    return {"pages": pages_out}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", choices=["content", "meta"])
    parser.add_argument("--selftest", action="store_true")
    parser.add_argument("path", nargs="?")
    args = parser.parse_args()

    if args.selftest:
        selftest()

    if not args.mode or not args.path:
        print("usage: pdf_extract.py --mode=content|meta <path>", file=sys.stderr)
        sys.exit(1)

    try:
        if args.mode == "meta":
            result = extract_meta(args.path)
        else:
            result = extract_content(args.path)
        print(json.dumps(result, separators=(",", ":")))
    except Exception as e:
        print(f"error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
