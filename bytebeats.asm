l6:
    copy r15 r2
    jo r6 l7
    rotlimm r15 r15 1
    jo r6 l8
    rotlimm r15 r15 1
    jo r6 l9
    rotlimm r15 r15 1
    jo r6 l10
    rotlimm r15 r15 1
    output r11
    output r11
    output r11
    output r11
    output r11
l7:
    output r11
    output r11
    output r11
    output r11
l8:
    output r11
    output r11
    output r11
l9:
    output r11
    output r11
l10:
    output r11
l2:
l5:
    output r11
    outputw r5
;    addmimmw r5 r5 3
;    submimmw r5 r5 110
    addcimm r11 r11 1
    output r11
    output r9
    jo r11 l5
    jo r4 l3
l0:
    addcimm r11 r11 3
    addmimmw r8 r8 1
    addmw r0 r3
    addm r1 r17
    addcimm r11 r11 1
    addmimm r0 r0 5
    rotlimmw r0 r0 0
    addmimm r0 r0 1
    output r11
    output r11
    rotrimmw r0 r0 2
l1:
    addmimm r0 r0 4
    addmw r1 r0
    subm r2 r3
    addc r2 r3
    addcimm r3 r3 1
    subcimmw r1 r1 0
    rotlimmw r1 r1 4
    jo r2 l1
    shl r1 r2
    shl r1 r1
    rotrimmw r1 r1 4
    jo r1 l6
    addmw r5 r1
    addmw r5 r0
    jo r10 l5
    rotlimm r1 r1 31
    addmw r5 r1
    shrimmw r5 r5 4
    addm r10 r1
    rotlimmw r5 r5 19
    jo r10 l1
    rotlimm r0 r0 1
;    addmimm r0 r0 127
    output r11
    output r11
    output r10
    output r10
;    jo r10 l3
    output r10
    outputw r5
    output r3
    output r4
l3:
    output r10
    xorimm r10 r10 1

    rotlimm r1 r1 4

    numzerosw r6 r2
    addmw r8 r6
    numzeros r13 r3
    addcw r6 r7
    rotrw r5 r5
    addc r10 r2
    jo r10 l0
    xor r13 r13
    addmw r1 r6
    rotlimmw r1 r1 13
    jo r2 l4
    rotrimmw r1 r1 13
    rotl r1 r0
    addcimm r2 r2 1

    xor r2 r3
    output r2
l4:
    shlimmw r0 r0 5
    jmp l0

