    addmimm r0 r0 3
    addmimm r1 r1 0
    addmimm r2 r2 13
    addmimm r3 r3 111
    addmimm r16 r16 127
l2:
    addmimm r17 r17 1
l0:
    addmw r0 r3
    addm r1 r17
l1:
    addmimm r0 r0 5
    rotlimmw r0 r0 2
    shrimmw r0 r0 4
    addmimm r0 r0 4
    addmw r1 r0
    subm r2 r3
    addc r2 r3
    addcimm r3 r3 1
    subcimmw r1 r1 0
    rotlimmw r1 r1 4
    jo r2 l1
    rotrimmw r1 r1 4
    jo r3 l4
    addmw r5 r1
    jo r10 l2
    rotlimm r1 r1 31
    addmw r5 r1
    shrimmw r5 r5 4
    rotlimmw r5 r5 19
    output r11
    output r11
    jo r4 l3
    output r11
l3:
    output r10

    rotlimm r1 r1 4

    numzerosw r6 r2
    addmw r8 r6
    numzeros r13 r3
    addc r12 r13
    xor r13 r13
    addmw r1 r6
    rotlimmw r1 r1 13
    jo r2 l0
    rotrimmw r1 r1 13
    rotl r1 r0
    addcimm r2 r2 1

    rotrimm r1 r1 3

    shrimmw r3 r3 1
    mulcimmw r3 r3 7
    xor r1 r4
    xor r2 r3
    output r2
    jmp l0

l4:
    mulc r2 r2
    addcw r0 r7
    addmimmw r0 r0 1
    jmp l0

