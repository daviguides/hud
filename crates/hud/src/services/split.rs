//! A layout spec to styled runs: the terminal divided into regions by sizes and ratios, each leaf
//! rendered into its region, the regions stitched back into whole lines.

use super::frame::{run, split_lines};
use crate::model::{Capabilities, Layout, Segment, Splitter, Style};

/// What a child asks of the space it is dividing.
#[derive(Clone, Copy)]
pub(crate) struct Edge {
    size: Option<usize>,
    ratio: usize,
    minimum: usize,
}

/// Divides `total` cells among `edges` so that fixed sizes are kept, a flexible edge is never
/// below its minimum, and the rest is shared by ratio, the remainders carried to the next edge.
/// The sizes add up to `total` unless the constraints cannot all be met.
pub(crate) fn ratio_resolve(total: usize, edges: &[Edge]) -> Vec<usize> {
    let mut sizes: Vec<Option<usize>> = edges
        .iter()
        .map(|edge| edge.size.filter(|&size| size != 0))
        .collect();
    while sizes.iter().any(Option::is_none) {
        let flexible: Vec<usize> = sizes
            .iter()
            .enumerate()
            .filter(|(_, size)| size.is_none())
            .map(|(index, _)| index)
            .collect();
        let fixed: usize = sizes.iter().map(|size| size.unwrap_or(0)).sum();
        let remaining = total as i128 - fixed as i128;
        if remaining <= 0 {
            return sizes
                .iter()
                .zip(edges)
                .map(|(size, edge)| {
                    size.unwrap_or_else(|| if edge.minimum == 0 { 1 } else { edge.minimum })
                })
                .collect();
        }
        let denominator: i128 = flexible
            .iter()
            .map(|&index| if edges[index].ratio == 0 { 1 } else { edges[index].ratio } as i128)
            .sum();
        let below_minimum = flexible.iter().copied().find(|&index| {
            remaining * edges[index].ratio as i128 <= edges[index].minimum as i128 * denominator
        });
        match below_minimum {
            Some(index) => sizes[index] = Some(edges[index].minimum),
            None => {
                let mut carried: i128 = 0;
                for &index in &flexible {
                    let share = remaining * edges[index].ratio as i128 + carried;
                    sizes[index] = Some((share / denominator) as usize);
                    carried = share % denominator;
                }
                break;
            }
        }
    }
    sizes.into_iter().map(|size| size.unwrap_or(0)).collect()
}

/// A rectangle of the terminal.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Region {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

fn visible(layout: &Layout) -> Vec<&Layout> {
    layout
        .children
        .iter()
        .filter(|child| child.visible)
        .collect()
}

fn divide<'a>(
    splitter: Splitter,
    children: &[&'a Layout],
    region: Region,
) -> Vec<(&'a Layout, Region)> {
    let edges: Vec<Edge> = children
        .iter()
        .map(|child| Edge {
            size: child.size,
            ratio: child.ratio,
            minimum: child.minimum_size,
        })
        .collect();
    let mut offset = 0;
    let mut out = Vec::with_capacity(children.len());
    match splitter {
        Splitter::Row => {
            for (child, width) in children.iter().zip(ratio_resolve(region.width, &edges)) {
                out.push((
                    *child,
                    Region {
                        x: region.x + offset,
                        width,
                        ..region
                    },
                ));
                offset += width;
            }
        }
        Splitter::Column => {
            for (child, height) in children.iter().zip(ratio_resolve(region.height, &edges)) {
                out.push((
                    *child,
                    Region {
                        y: region.y + offset,
                        height,
                        ..region
                    },
                ));
                offset += height;
            }
        }
    }
    out
}

/// The lines of one leaf: exactly `region.height` lines of exactly `region.width` cells.
fn leaf_lines(layout: &Layout, region: Region, caps: &Capabilities) -> Vec<Vec<Segment>> {
    let null = Style::new();
    let blank = || {
        if region.width == 0 {
            Vec::new()
        } else {
            vec![run(" ".repeat(region.width), &null)]
        }
    };
    let mut lines = match &layout.body {
        Some(body) if region.width > 0 => {
            let segments = body.0.render_region(region.width, region.height, caps);
            split_lines(segments, region.width, Some(&null))
        }
        _ => Vec::new(),
    };
    lines.truncate(region.height);
    while lines.len() < region.height {
        lines.push(blank());
    }
    lines
}

/// Renders `layout` into a `width` by `height` terminal: `height` lines, each ended by a newline.
pub(crate) fn render_layout(
    layout: &Layout,
    width: usize,
    height: usize,
    caps: &Capabilities,
) -> Vec<Segment> {
    let mut stack = vec![(
        layout,
        Region {
            x: 0,
            y: 0,
            width,
            height,
        },
    )];
    let mut regions: Vec<(&Layout, Region)> = Vec::new();
    while let Some((current, region)) = stack.pop() {
        regions.push((current, region));
        let children = visible(current);
        if !children.is_empty() {
            stack.extend(divide(current.splitter, &children, region));
        }
    }
    regions.sort_by_key(|&(_, region)| region);

    let mut rows: Vec<Vec<Segment>> = vec![Vec::new(); height];
    for (leaf, region) in regions {
        if !visible(leaf).is_empty() {
            continue;
        }
        let lines = leaf_lines(leaf, region, caps);
        for (row, line) in rows
            .iter_mut()
            .skip(region.y)
            .take(region.height)
            .zip(lines)
        {
            row.extend(line);
        }
    }
    let null = Style::new();
    let mut out = Vec::new();
    for row in rows {
        out.extend(row);
        out.push(run("\n", &null));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edge(size: Option<usize>, ratio: usize, minimum: usize) -> Edge {
        Edge {
            size,
            ratio,
            minimum,
        }
    }

    #[test]
    fn flexible_edges_share_by_ratio_and_carry_the_remainder() {
        let sizes = ratio_resolve(110, &[edge(None, 1, 1), edge(None, 1, 1), edge(None, 1, 1)]);
        assert_eq!(sizes, [36, 37, 37]);
        assert_eq!(sizes.iter().sum::<usize>(), 110);
        assert_eq!(
            ratio_resolve(10, &[edge(None, 1, 1), edge(None, 2, 1)]),
            [3, 7]
        );
    }

    #[test]
    fn a_fixed_size_is_kept_and_the_rest_is_shared() {
        assert_eq!(
            ratio_resolve(20, &[edge(Some(5), 1, 1), edge(None, 1, 1)]),
            [5, 15]
        );
        assert_eq!(
            ratio_resolve(20, &[edge(Some(0), 1, 1), edge(None, 1, 1)]),
            [10, 10]
        );
    }

    #[test]
    fn an_edge_below_its_minimum_takes_the_minimum_first() {
        assert_eq!(
            ratio_resolve(30, &[edge(None, 1, 20), edge(None, 1, 1)]),
            [20, 10]
        );
    }

    #[test]
    fn no_room_gives_the_flexible_edges_their_minimum() {
        assert_eq!(
            ratio_resolve(10, &[edge(Some(10), 1, 1), edge(None, 1, 4)]),
            [10, 4]
        );
    }
}
