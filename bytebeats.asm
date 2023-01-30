l6:
    subc r0 r11

    addm r14 r0
    rotlimm r14 r14 2
    copy r15 r14
    addm r15 r3


    jo r15 l10
    mod r11 r12
    rotlimm r15 r15 1
    jo r15 l5
    rotlimm r15 r15 1
    jo r15 l8
    subm r2 r11

l12:
    addmw r5 r0
    output r11
    output r11
    output r11
    output r11
    output r11
    output r11
    output r11
    output r11

    addmimmw r0 r0 8
    jo r1 l6

l8:
    addm r11 r1
    addm r11 r0
    output r11
    output r11
    output r11
    output r11
l9:
;    addmw r5 r0
    output r11
    output r11
l10:
;    addmw r5 r3
    output r11
    output r11
    output r11
    output r11
l2:
;    output r11
;    addcimm r11 r11 1
    output r11
    output r9
    jo r11 l5
    jo r4 l3
l0:
    addcimm r11 r11 3
l5:
    addmimmw r8 r8 1
    addmw r0 r3
    addm r1 r17
    addcimm r11 r11 1
    addmimm r0 r0 5
    rotlimmw r0 r0 0
    addmimm r0 r0 1
;    output r11
;    output r11
    rotrimmw r0 r0 2
l1:
    addmimm r0 r0 4
    addmw r1 r0
    subm r2 r3
    addc r2 r3
    addcimm r3 r3 1
    subcimmw r1 r1 0
    rotlimmw r1 r1 4
;    jo r3 l11
    shl r1 r2
    shl r1 r1
    rotrimmw r1 r1 4
    jo r3 l11
    jmp l12
l11:
    addmw r5 r1
    addmw r5 r0
    jo r10 l5
    rotlimm r1 r1 31
    addmw r5 r1
    shrimmw r5 r5 4
    addm r10 r1
    rotlimmw r5 r5 19
    jo r10 l12
;    rotlimm r0 r0 1
    jo r7 l12
;    jmp l12
l3:
    xorimm r10 r10 3

    xorimm r3 r3 3
    shrimmw r1 r1 3
    jo r2 l6
    rotlimmw r1 r1 3
;    rotlimmw r1 r1 1
    jmp l12
