* enemy types
    * ghost
    * shooter
    * boss
    * swarm

* vignette, light shit
    * maybe just per pixel, not that high res but interpolated / smoothed and like breathing a big

* high res terrain

* extend terrain

* add the altars at each level that reveal the stairs

* when you died, "you died" screen


Classes
    * paladin - i will vanquish evil
        * holy light - flexible laser spell
        * shoot 3 crosses they get stuck in walls
        * some kind of melee attack
        * proc on damage
    * demonologist - down there for selfish reasons

    or should it just be freeform

* abilities come from items
* progression is mostly skill and finding a good combination of items
* every run is different


make a variety of enemy types
each level you could populate the enemies with a table

be cool to have a creepy paragraph about the floor ur about to go down to
that was based on what it had
eg you feel a creeping feeling of dread come over you
the temperature plummets and you have begun to shiver

the sound of scraping
etc


have a purple guy that shoots projectiles that shoot projectiles
have a guy that shoots (summons) guys
if it gets too tedious to do the entity data it could be just populating vectors

-----

ENEMY REPOSITORY
OTHER PLAYER ATTACKS
VIGNETTE


ITEMS
4 slots
+ maybe tab to swap the 4
inventory is 12: 4 actives, 4 swap in and 4 extra / misc
health potion would be good
items can just have a function pointer i guess
or maybe we will have an omega switch statement
probably have modifiers and allow transfer of modifiers


shield
detect life
amulet: get rezzed

yeah fuck laser would be cool if it was 3 lasers


think historia civilis stylized graphics

SDFs: the minimum distance is the minimum of the distances to all the components
but only need to check nearby components

imagine how good fixing the bugs and stuff in the shadow will be
have soft shadows, blend em in

closed form of the distance wouldnt even be hard
min of distance of line segments, + d for thickness
then just warp domain as see fit

an enemy who patrols the map
an enemy who leaves behind stationary things
it really is as if when there are 2 enemies it will choose to damage the rear one

definitely could do procedural moss and grass and shit off this sdf thing
the current glitch kind of looks like it

imagine simulating explosion gas and channeling lol

youve got to be an alchemist who stumbled into another dimension
an occult dimension

like if we know distance to wall, normal for each point...

we can know direction to surface as well
yea like when we have normals for all the inside points collision will be good

they do the decompose into rects approach
i like mine

bro I already know where all the onscreen walls are - onscreen
can definitely do some buffer shit



degenerate building to get ungodly far
combine items so they have triggered effects that they can reset on one another

map: greater open chance but + other shit as obstructions

contractual modifiers, gain a stack of whatever when an enemy is killed, when you take damage, 

sus out noita
um yea maybe more generally there needs to be some kind of lighting pass. emission maps
it should probably be on gpu
could have a specular map or something
ideal if i could have the same surface on gpu and on cpu side

---

add decrease key to PQ
then add the pq based distance marching thing and time the speedup

it will be better for the AI if they avoid terrain as well

correlate wandering of mobs
move faster the more they are

i understand the 1d 1d maybe
push it along
