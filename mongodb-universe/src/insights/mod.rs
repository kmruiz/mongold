pub enum ClusterInsight {
    Version,
    Sharded,
    Atlas,
}
pub enum DatabaseInsight {
    Sharded,
}
pub enum CollectionInsight {
    Sharded,
    RegularIndexes,
    SearchIndexes,
    Capped,
    TimeSeries,
}
