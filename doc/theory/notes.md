# Fusion

FIXME fusor decomposition graph of a fuset _&phi;_,
<tt>FD</tt>(_&phi;_).

FIXME fusibles are &ldquo;accessible&rdquo;, in the sense

# Tightness

All subsets of any tight fuset are tight.  Therefore, given any
tight fuset _&phi;_ and any fuset _&psi;_, their intersection and
set difference, _&phi;_ &setminus; _&psi;_ are tight (absorption).
FIXME However, their union may not be tight (no closure).

FIXME All proper tight fusets are thin.

FIXME Minimal tight union of maximal stars in a fuset _&phi;_.

FIXME Two lower (upper) stars are _colliding_ iff their frames
overlap: the frames are intersecting, yet incomparable by
inclusion.

FIXME Maximal tight subset of a fuset _&phi;_.

### FIXME condition

for every dipole _{(x, u), (v, y)}_ _x_ is in _v_ and y is
in u.

FIXME necessary: the pre-set is fork-armed by the post-set, which
is join-armed by the pre-set.  For every pre-set dot _x_ and
every fork (also _x_-tipped?) _x_ is in the pit and for every
post-set dot _y_ and every join _y_ is in the pit.

FIXME Sufficient in a singular tight fuset.

### Tightness and graphs

FIXME ties and t-graphs: bipartite graph linking tied wedges.

FIXME d-graphs linking tight diodes having at least one comparable
wedge.

# Coherence

Given any two coherent fusets, their union is a coherent fuset.
FIXME However, neither intersection, nor set difference has to be.

Given a fuset _&phi;_ over ***X*** and two elements _x_ and _y_ of
_X, we say_ that

* _y_ is an _upper misstip_ of _x_ iff _y_ is fork-armed by _x_ but
  _x_ isn't join-armed by _y_;

* _y_ is a _lower misstip_ of _x_ iff _y_ is join-armed by _x_ but _x_
  isn't fork-armed by _y_;

* _y_ _weakly follows_ _x_ iff _y_ is join-armed by _x_ or _x_ is
  fork-armed by _y_, but not both; synonymously, _y_ is a _weak
  follower_ of _x_;

* _y_ _strongly follows_ _x_ iff _y_ is join-armed by _x_ and _x_
  is fork-armed by _y_; synonymously, _y_ is a _strong follower_
  of _x_.

As may be shown by examination of the domain partition table, the
exclusion of weak followers is equivalent to coherence (and so is the
exclusion of misstips).

A _strong patching_ of a dot _x_ in a fuset _&phi;_ is a pair of
domain subsets (<tt><b>P</b></tt><sub>_&phi;_</sub>(_x_),
<tt><b>p</b></tt><sub>_&phi;_</sub>(_x_)), where the components
<tt><b>P</b></tt><sub>_&phi;_</sub>(_x_)&nbsp;=&nbsp;{_y_ | _x_ &larr;<sub>_&phi;_</sub> _y_ &and; _y_ &rarr;<sub>_&phi;_</sub> _x_} and
<tt><b>p</b></tt><sub>_&phi;_</sub>(_x_)&nbsp;=&nbsp;{_y_ | _x_ &rarr;<sub>_&phi;_</sub> _y_ &and; _y_ &larr;<sub>_&phi;_</sub> _x_} are the _upper_ and _lower strong patching_ of _x_ in _&phi;_.

A _weak patching_ of a dot _x_ in a fuset _&phi;_ is a 4-tuple
of domain subsets (<tt><b>A</b></tt><sub>_&phi;_</sub>(_x_),
<tt><b>a</b></tt><sub>_&phi;_</sub>(_x_),
<tt><b>T</b></tt><sub>_&phi;_</sub>(_x_), <tt><b>t</b></tt><sub>_&phi;_</sub>(_x_)), where the components
<tt><b>A</b></tt><sub>_&phi;_</sub>(_x_)&nbsp;=&nbsp;{_y_ | _x_ &larr;<sub>_&phi;_</sub> _y_ &and; _y_ &nrarr;<sub>_&phi;_</sub> _x_},
<tt><b>a</b></tt><sub>_&phi;_</sub>(_x_)&nbsp;=&nbsp;{_y_ | _x_ &rarr;<sub>_&phi;_</sub> _y_ &and; _y_ &nlarr;<sub>_&phi;_</sub> _x_},
<tt><b>T</b></tt><sub>_&phi;_</sub>(_x_)&nbsp;=&nbsp;{_y_ | _y_ &rarr;<sub>_&phi;_</sub> _x_ &and; _x_ &nlarr;<sub>_&phi;_</sub> _y_}, and
<tt><b>t</b></tt><sub>_&phi;_</sub>(_x_)&nbsp;=&nbsp;{_y_ | _y_ &larr;<sub>_&phi;_</sub> _x_ &and; _x_ &nrarr;<sub>_&phi;_</sub> _y_},
are the _upper_ and _lower arm patching_ of _x_ in _&phi;_ (the
dots arming _x_), and the _upper_ and _lower tip patching_ of _x_
in _&phi;_ (the tips armed by _x_).

The following _weak patching pattern table_ lists possible
component patterns of weak patching of a dot, depending on its
class.  Non-empty components are represented by the corresponding
letters: <tt><b>A</b></tt>, <tt><b>a</b></tt>, <tt><b>T</b></tt>,
and <tt><b>t</b></tt>.  Empty components are omitted, unless all
four components are empty, in which case the pattern is
represented by the symbol <tt>&empty;</tt>.

| <p align="right">_arm_</p>_tip_ | <center>not an arm<br>(not in span)</center> | <center>join's arm<br>(in over-set)</center> | <center>fork's arm<br>(in under-set)</center> | <center>both<br>(in co-interior)</center> |
|-----------------------------------------|:----:|:----------------------:|:----------------------:|:----------------------:|
| **not a tip**<br>**(not in carrier)**  | <tt>&empty;</tt> | <tt><b>t</b></tt> | <tt><b>T</b></tt> | <tt><b>Tt</b></tt> |
| **fork's tip**<br>**(in pre-set)**  | <tt><b>a</b></tt> | <tt><b>ae</b></tt>, <tt><b>a</b></tt>, <tt><b>t</b></tt>, <tt>&empty;</tt> | <tt><b>aT</b></tt> | <tt><b>aTt</b></tt>, <tt><b>aT</b></tt>, <tt><b>Tt</b></tt>, <tt><b>T</b></tt> |
| **join's tip**<br>**(in post-set)** | <tt><b>A</b></tt> | <tt><b>At</b></tt> | <tt><b>AT</b></tt>, <tt><b>A</b></tt>, <tt><b>T</b></tt>, <tt>&empty;</tt> | <tt><b>ATt</b></tt>, <tt><b>At</b></tt>, <tt><b>Tt</b></tt>, <tt><b>t</b></tt> |
| **both**<br>**(in interior)**           | <tt><b>Aa</b></tt> | <tt><b>Aat</b></tt>, <tt><b>Aa</b></tt>, <tt><b>At</b></tt>, <tt><b>A</b></tt> | <tt><b>AaT</b></tt>, <tt><b>Aa</b></tt>, <tt><b>aT</b></tt>, <tt><b>a</b></tt> | any pattern |

### Set equality condition

FIXME By analogy to tightness condition... its pre-set is
fork-framed by its post-set, which is join-framed by the pre-set
&mdash; and this is equivalent to the inclusion of over-set in
pre-set and under-set in post-set.  The necessary condition for a
fuset to be coherent is even stronger: it is necessary that
pre-set is equal to over-set and post-set is equal to under-set.
This is the _set equality condition_ of coherence.

To see that this is the case, note that if a pre-set dot _x_
isn't in the over-set, then _x_ is fork-armed by at least one arm
and all such arms are weak followers of _x_; conversely, if an
over-set dot _x_ isn't in the pre-set, then there is at least
one tip join-armed by _x_ and all such tips are weak followers of
_x_.  Hence, if a fuset doesn't contain weak followers, then its
pre-set and over-set must be equal.  The other equality is
supported by the symmetric argument.

However, set equality isn't a sufficient condition of strength.
For example, it is possible to find a coherent fuset _&phi;_ over
some domain ***X***, and a fork (_x_, _u_) over ***X***, not a
member of _&phi;_, such that _x_ is in <tt>Pre</tt>(_&phi;_), _u_
is a subset of <tt>Under</tt>(_&phi;_), but not a subset of
<tt>Under</tt>(_x_<sub>&star;</sub>(_&phi;_)).  Then, if _&phi;_
is augmented with such a fork to form a fuset _&psi;_, the set
equality condition is satisfied by _&psi;_, but _&psi;_ isn't
coherent, because any element of _u_ not in
<tt>Under</tt>(_x_<sub>&star;</sub>(_&phi;_)) is a weak follower
of _x_ in _&psi;_.

### Fusing complement (padding, promotion) and coherent closure

Given a domain ***X*** and two fusets _&phi;_ and _&psi;_ over ***X***
we define:

*  

FIXME relative complement in a coherent superset (or in a coherent
closure)

FIXME coherent closure: _&psi;_ is the maximal fuset over ***X***.

for all misstips (_x_, <tt><b>t</b></tt><sub>_&phi;_</sub>(_x_))
(<tt><b>T</b></tt><sub>_&phi;_</sub>(_x_), _x_)

# Product fusets

FIXME parallel multiplication.

# Fuset profiles

FIXME additive profile, multiplicative profile

# Flow paths and tracks

### Flow paths

Given elements _s_ and _t_ of ***X***, a _flow path_ _&pi;_ from
_s_ to _t_ is a minimal non-empty fuset over ***X***, such that
the post-set of _&pi;_ is fork-reachable from _s_ in _&pi;_ and
the pre-set of _&pi;_ is join-reachable from _t_ in _&pi;_.

> _s_ &Rarr;<sub>_&pi;_</sub> <tt>Post</tt>(_&pi;_) &and; _t_ &Larr;<sub>_&pi;_</sub> <tt>Pre</tt>(_&pi;_)

A fuset _&phi;_ over ***X*** forms a _flow cycle_ iff there is an
_x_ in ***X*** such that _&phi;_ is a flow path from _x_ to _x_.

FIXME Given a path _&pi;<sub>1</sub>_ from _s_ to _r_ and a path
_&pi;<sub>2</sub>_ from _r_ to _t_

FIXME If a dot _r_ is internal in a path _&pi;_ from _s_ to _t_

FIXME all flow paths are singular.

FIXME a fuset is a thin path iff it is a minimal tight fuset.
If a flow path isn't thin, then it is not tight.

FIXME It is convenient to say _thin path_ instead of _thin flow
path_.

### Stems

Given a fuset _&phi;_ over ***X***, and two subsets ***Y***,
***Z*** of ***X***, ***Z*** is _hyper-connected_ to ***Y*** in
_&phi;_ iff any element of ***Z*** is fork-reachable from some
element of ***Y*** and any element of ***Y*** is join-reachable
from some element of ***Z***.

Given subsets ***S*** and ***T*** of ***X***, a _stem_ from
***S*** to ***T*** is a fuset _&sigma;_ over ***X*** such that
the post-set of _&sigma;_ is hyper-connected to ***S*** in
_&sigma;_ and the pre-set of _&sigma;_ is hyper-connected to
***T*** in _&sigma;_.  A stem is _cycle-free_ iff it doesn't
contain any flow cycles.

FIXME Note, that all singular coherent stems are minimal, i.e. one
such stem never includes another as a proper subset.

FIXME A singular coherent thin stem is called a _floret_.

FIXME The _flower_ of a fuset _&phi;_ is the set of all florets
of _&phi;_ and the _core_ of _&phi;_ is their union.

FIXME A _weed_ is a fuset without florets.

FIXME not all florets are tight.

FIXME a set of florets is co-firable iff its union is singular.

# Core net

The set <tt>Flow</tt>(***X***) of all florets over some domain
***X*** will be called the _flower_ of ***X***.  Similarly, the
set <tt>Flow</tt>(_&phi;_) of all florets included in a fuset
_&phi;_ will be called the _flower_ of _&phi;_.

The _core net_ of a domain ***X*** is the bipartite digraph linking an
element _x_ of ***X*** to a floret _&gamma;_ in <tt>Flow</tt>(***X***)
iff _x_ is in the pre-set of _&gamma;_, and linking a floret _&delta;_
in <tt>Flow</tt>(***X***) to an element _y_ of ***X*** iff _y_ is in
the post-set of _&delta;_.  The _carrier_ of the core net of ***X***
is the union of carriers of all florets in <tt>Flow</tt>(***X***)
&mdash; the set of all elements of ***X*** which aren't isolated in
the graph.  The _span_ of a core net is the neighborhood of its
carrier.

The core net of a fuset _&phi;_ over ***X*** is the restriction
of the core net of ***X*** to the flower of _&phi;_, i.e. the
subgraph induced by the union of ***X*** and
<tt>Flow</tt>(_&phi;_).

FIXME reachability, 

FIXME juncture, junction, (a -> x y, x <- a b) => {(a,x)},{(a,y)},{(x,b)}

FIXME push-pull derivation
