use bigint::H256;

mod iommu;

fn main() {
    iommu::output(H256([1; 32]));
}
