"""Pure proposed index-admission planner; no store changes or expanded workloads."""
from dataclasses import dataclass


class Refusal(ValueError):pass


@dataclass(frozen=True)
class Proposal:
    entries:int=60
    leaves:int=4096
    depth:int=24
    splits:int=8

    def __post_init__(self):
        if not 1<=self.entries<=60 or not 1<=self.leaves<=4096 or not 1<=self.depth<=24 or not 1<=self.splits<=8:
            raise Refusal('outside proposed resource profile')


def plan(bucket,keys,incoming,current_leaves,proposal=Proposal()):
    """Stage one scoped key into at most 61 entries; return costs, never perform IO."""
    if not isinstance(bucket,int) or bucket<1 or not 1<=current_leaves<=proposal.leaves:raise Refusal('invalid current shape')
    depth=bucket.bit_length()-1;prefix=bucket-(1<<depth)
    if depth>proposal.depth or len(keys)>proposal.entries:raise Refusal('existing node exceeds contract')
    allkeys=list(keys)+[incoming]
    if any(not isinstance(key,bytes) or len(key)!=32 for key in allkeys):raise Refusal('key width')
    if len(set(allkeys))!=len(allkeys):raise Refusal('already bound/duplicate key')
    if any((int.from_bytes(key,'big')>>(256-depth) if depth else 0)!=prefix for key in allkeys):raise Refusal('wrong leaf interval')
    splits=0;leaves=[]
    def visit(address,items):
        nonlocal splits
        if len(items)<=proposal.entries:leaves.append((address,len(items)));return
        d=address.bit_length()-1
        if d>=proposal.depth:raise Refusal('depth budget')
        if splits>=proposal.splits:raise Refusal('split budget')
        if current_leaves+splits>=proposal.leaves:raise Refusal('leaf budget')
        splits+=1;children=[[],[]]
        for key in items:children[(int.from_bytes(key,'big')>>(255-d))&1].append(key)
        visit(address*2,children[0]);visit(address*2+1,children[1])
    visit(bucket,allkeys)
    # One insertion cannot overflow both children: at most 61 entries enter a split.
    if splits and (len(allkeys)!=proposal.entries+1 or depth+splits>proposal.depth):raise AssertionError('single insertion bound')
    branch_writes=depth+splits;leaf_writes=len(leaves)
    return {'splits':splits,'new_leaves':current_leaves+splits,'node_writes':branch_writes+leaf_writes,
            'logical_node_bytes':65*branch_writes+sum(1+64*count for _,count in leaves),
            'packed_node_dml_statements':2*branch_writes+leaf_writes,'leaf_counts':leaves}


def bounds(proposal=Proposal()):
    possibilities=[]
    for depth in range(proposal.depth+1):
        for splits in range(min(proposal.splits,proposal.depth-depth)+1):
            entries=proposal.entries+int(bool(splits));leaf_writes=splits+1
            possibilities.append({'nodes':depth+2*splits+1,'bytes':65*(depth+splits)+64*entries+leaf_writes,
                                  'dml':2*depth+3*splits+1})
    return {'configuration':proposal.__dict__,'structural_entry_ceiling':proposal.leaves*proposal.entries,
        'current_node_ceiling':2*proposal.leaves-1,'address_ceiling':(1<<(proposal.depth+1))-1,
        'authentication_node_bytes_at_full_capacity':proposal.leaves*(1+proposal.entries*64)+(proposal.leaves-1)*65,
        'proof_nodes':proposal.depth+1,'proof_node_bytes':proposal.depth*65+proposal.entries*64+1,
        'publication_node_writes':max(p['nodes'] for p in possibilities),
        'publication_node_bytes':max(p['bytes'] for p in possibilities),
        'publication_index_bytes_with_64B_envelope':max(p['bytes'] for p in possibilities)+64,
        'packed_node_dml_statements':max(p['dml'] for p in possibilities),
        'not_bounds':['Python/native heap peak','SQLite dirty pages/WAL frames','G2 payload bytes','physical device writes']}


if __name__=='__main__':
    import json
    print(json.dumps(bounds(),indent=2))
