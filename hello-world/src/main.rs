mod iommu;

#[derive(Eq, PartialEq, Hash)]
pub struct H256(pub [u8; 32]);

fn main() {
    iommu::output(H256([1; 32]));
}
