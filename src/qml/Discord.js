.pragma library

// Discord's default avatar color palette
var discordAvatarColors = [
    "#5865F2", // blurple
    "#747F8D", // gray
    "#3BA55C", // green
    "#FAA61A", // yellow
    "#ED4245", // red
    "#EB459E"  // fuchsia
];

function avatarColor(str) {
    if (!str || str.length === 0) return discordAvatarColors[0];
    var hash = 0;
    for (var i = 0; i < str.length; i++)
        hash = ((hash << 5) - hash + str.charCodeAt(i)) | 0;
    return discordAvatarColors[Math.abs(hash) % discordAvatarColors.length];
}

function twemojiUrl(emoji) {
    var cp = [];
    var hasZwj = emoji.indexOf('\u200D') >= 0;
    for (var i = 0; i < emoji.length; i++) {
        var c = emoji.charCodeAt(i);
        if (c >= 0xD800 && c <= 0xDBFF && i + 1 < emoji.length) {
            var lo = emoji.charCodeAt(i + 1);
            if (lo >= 0xDC00 && lo <= 0xDFFF) {
                cp.push((0x10000 + ((c - 0xD800) << 10) + (lo - 0xDC00)).toString(16));
                i++;
                continue;
            }
        }
        if (!hasZwj && c === 0xFE0F) continue;
        cp.push(c.toString(16));
    }
    return "https://cdn.jsdelivr.net/gh/jdecked/twemoji@latest/assets/72x72/" + cp.join('-') + ".png";
}

function isEmojiCodepoint(cp) {
    if (cp === 0xFE0F || cp === 0x200D) return true;
    if (cp >= 0x1F300 && cp <= 0x1F9FF) return true;
    if (cp >= 0x1F600 && cp <= 0x1F64F) return true;
    if (cp >= 0x1F910 && cp <= 0x1F92F) return true;
    if (cp >= 0x1F000 && cp <= 0x1F02F) return true;
    if (cp >= 0x2600 && cp <= 0x26FF) return true;
    if (cp >= 0x2700 && cp <= 0x27BF) return true;
    if (cp >= 0x1F1E6 && cp <= 0x1F1FF) return true;
    if (cp >= 0x231A && cp <= 0x231B) return true;
    if (cp >= 0x23E9 && cp <= 0x23F3) return true;
    if (cp >= 0x23F8 && cp <= 0x23FA) return true;
    if (cp >= 0x25AA && cp <= 0x25AB) return true;
    if (cp >= 0x25B6 && cp <= 0x25B6) return true;
    if (cp >= 0x25C0 && cp <= 0x25C0) return true;
    if (cp >= 0x25FB && cp <= 0x25FE) return true;
    if (cp >= 0x2614 && cp <= 0x2615) return true;
    if (cp >= 0x2648 && cp <= 0x2653) return true;
    if (cp >= 0x267F && cp <= 0x267F) return true;
    if (cp >= 0x2693 && cp <= 0x2693) return true;
    if (cp >= 0x26A1 && cp <= 0x26A1) return true;
    if (cp >= 0x26AA && cp <= 0x26AB) return true;
    if (cp >= 0x26BD && cp <= 0x26BE) return true;
    if (cp >= 0x26C4 && cp <= 0x26C5) return true;
    if (cp >= 0x26CE && cp <= 0x26CE) return true;
    if (cp >= 0x26D4 && cp <= 0x26D4) return true;
    if (cp >= 0x26EA && cp <= 0x26EA) return true;
    if (cp >= 0x26F2 && cp <= 0x26F3) return true;
    if (cp >= 0x26F5 && cp <= 0x26F5) return true;
    if (cp >= 0x26FA && cp <= 0x26FA) return true;
    if (cp >= 0x26FD && cp <= 0x26FD) return true;
    if (cp >= 0x2702 && cp <= 0x2702) return true;
    if (cp >= 0x2705 && cp <= 0x2705) return true;
    if (cp >= 0x2708 && cp <= 0x270D) return true;
    if (cp >= 0x270F && cp <= 0x270F) return true;
    if (cp >= 0x2712 && cp <= 0x2712) return true;
    if (cp >= 0x2714 && cp <= 0x2714) return true;
    if (cp >= 0x2716 && cp <= 0x2716) return true;
    if (cp >= 0x271D && cp <= 0x271D) return true;
    if (cp >= 0x2721 && cp <= 0x2721) return true;
    if (cp >= 0x2728 && cp <= 0x2728) return true;
    if (cp >= 0x2733 && cp <= 0x2734) return true;
    if (cp >= 0x2744 && cp <= 0x2744) return true;
    if (cp >= 0x2747 && cp <= 0x2747) return true;
    if (cp >= 0x274C && cp <= 0x274C) return true;
    if (cp >= 0x274E && cp <= 0x274E) return true;
    if (cp >= 0x2753 && cp <= 0x2755) return true;
    if (cp >= 0x2757 && cp <= 0x2757) return true;
    if (cp >= 0x2763 && cp <= 0x2764) return true;
    if (cp >= 0x2795 && cp <= 0x2797) return true;
    if (cp >= 0x27A1 && cp <= 0x27A1) return true;
    if (cp >= 0x27B0 && cp <= 0x27B0) return true;
    if (cp >= 0x27BF && cp <= 0x27BF) return true;
    if (cp >= 0x2934 && cp <= 0x2935) return true;
    if (cp >= 0x2B05 && cp <= 0x2B07) return true;
    if (cp >= 0x2B1B && cp <= 0x2B1C) return true;
    if (cp >= 0x2B50 && cp <= 0x2B50) return true;
    if (cp >= 0x2B55 && cp <= 0x2B55) return true;
    if (cp >= 0x3030 && cp <= 0x3030) return true;
    if (cp >= 0x303D && cp <= 0x303D) return true;
    if (cp >= 0x3297 && cp <= 0x3297) return true;
    if (cp >= 0x3299 && cp <= 0x3299) return true;
    return false;
}

function segmentize(str) {
    if (!str || str.length === 0) return [];
    var segments = [];
    var i = 0;
    while (i < str.length) {
        var c = str.charCodeAt(i);
        var cp = c;
        var charLen = 1;
        if (c >= 0xD800 && c <= 0xDBFF && i + 1 < str.length) {
            var lo = str.charCodeAt(i + 1);
            if (lo >= 0xDC00 && lo <= 0xDFFF) {
                cp = 0x10000 + ((c - 0xD800) << 10) + (lo - 0xDC00);
                charLen = 2;
            }
        }
        if (isEmojiCodepoint(cp)) {
            var emojiEnd = i + charLen;
            if (emojiEnd < str.length && str.charCodeAt(emojiEnd) === 0xFE0F) emojiEnd++;
            while (emojiEnd < str.length) {
                var nc = str.charCodeAt(emojiEnd);
                var ncp = nc;
                var nlen = 1;
                if (nc >= 0xD800 && nc <= 0xDBFF && emojiEnd + 1 < str.length) {
                    var nlo = str.charCodeAt(emojiEnd + 1);
                    if (nlo >= 0xDC00 && nlo <= 0xDFFF) {
                        ncp = 0x10000 + ((nc - 0xD800) << 10) + (nlo - 0xDC00);
                        nlen = 2;
                    }
                }
                if (nc === 0x200D || ncp === 0xFE0F || isEmojiCodepoint(ncp)) {
                    emojiEnd += nlen;
                } else break;
            }
            segments.push({ type: "emoji", value: str.substring(i, emojiEnd) });
            i = emojiEnd;
            continue;
        }
        var textStart = i;
        while (i < str.length) {
            var c3 = str.charCodeAt(i);
            var cp3 = c3;
            var len3 = 1;
            if (c3 >= 0xD800 && c3 <= 0xDBFF && i + 1 < str.length) {
                var lo3 = str.charCodeAt(i + 1);
                if (lo3 >= 0xDC00 && lo3 <= 0xDFFF) {
                    cp3 = 0x10000 + ((c3 - 0xD800) << 10) + (lo3 - 0xDC00);
                    len3 = 2;
                }
            }
            if (!isEmojiCodepoint(cp3)) {
                i += len3;
            } else break;
        }
        if (i > textStart) {
            segments.push({ type: "text", value: str.substring(textStart, i) });
        }
    }
    return segments;
}
