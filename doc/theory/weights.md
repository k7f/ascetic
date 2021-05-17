# Two types of weight labeling

The implementation supports two different ways of weight-labeling of
c-e structures.  The purpose of both is to attach weights to arcs of a
core net, or equivalently &mdash; to nodes in firing components.

* Core-weight: individual weight label explicitly attached to an arc
  of a specific core net.

* Wedge-weight: generic weight of monomial causes or effects of a node.

  Wedge-weights represent the weight labeling used in the standard
  notation for polynomials.  Implementation-wise, they are attached to
  wedges (elements of a fuset: forks and joins) and inherited by
  core-weights of all corresponding arcs of the induced core net.

  Note that, in general, fork (join) pits are contained in, not equal
  to, post-sets (pre-sets) of the corresponding florets.

# Behavior

A finite weight of a node _x_ occurring in the pre-set of a transition
_t_ indicates the number of tokens to take out of node _x_ if
transition _t_ fires.  A finite weight of a node _x_ occurring in the
post-set of a transition _t_ indicates the number of tokens to put
into node _x_ if transition _t_ fires.

The finite positive weights also imply constraints a state must
satisfy for a transition to fire: the minimal number of tokens to be
supplied by pre-set nodes, so that token transfer may happen, and the
maximal number of tokens in post-set nodes, so that node capacities
aren't exceeded.

If weight zero is attached to a pre-set node _x_ of a transition _t_,
then tokens aren't taken out of _x_ when _t_ fires.  However,
transition _t_ never fires, unless there is a token in node _x_.
Another special case is the _&omega;_ (infinite) weight attached to a
pre-set node, which indicates that the presence of a token in that
node inhibits firing.
