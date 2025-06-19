//! Adhoc Tree Zipper implementation. See [Zippers](https://wiki.haskell.org/index.php?title=Zipper) for more information.

use std::collections::VecDeque;

// TODO:
// Is there a way to uniformly allocate `Node` ?
#[derive(Clone, Debug)]
pub enum Tree<T> {
    Leaf(T),
    Node(T, Vec<Tree<T>>),
}

impl<T> Tree<T> {
    pub fn iter_breadth(&self) -> BreadthFirstIterator<T> {
        BreadthFirstIterator::new(self)
    }
    pub fn iter_depth(&self) -> DepthFirstIterator<T> {
        DepthFirstIterator::new(self)
    }
}

impl<T: Clone> Tree<T> {
    /// Iterate with Zippers. The Zippers allow users to inspect the surrounding structure at
    /// the given Node.
    pub fn zipper_iter(&self) -> BreadthFirstTreeZipperIterator<T> {
        BreadthFirstTreeZipperIterator::new((*self).clone())
    }
}

// TODO:
// Make `item` take a reference, rather than owned.
/// Crumbs are used for traversing the structure.
#[derive(Clone, Debug)]
pub struct TreeCrumb<T> {
    item: T,
    // Left of the node
    left: Vec<Tree<T>>,
    // Right of the node
    right: Vec<Tree<T>>,
}

// NOTE:
// From LYaH, the Zipper is implemented using a list, where we PREPEND crumbs to the list of
// crumbs (Like a QUEUE).
// However, in Rust, we will use a Vec to work like a STACK, where crumbs will be "pushed" onto the
// stack.
#[derive(Debug)]
pub struct TreeZipper<T>(pub Tree<T>, pub Vec<TreeCrumb<T>>);

impl<T> TreeZipper<T> {
    /// Goes up one crumb. Think of it as retracing a step taken to get to this point in the
    /// structure.
    pub fn go_up(self) -> Option<Self> {
        let TreeZipper(focused, mut crumbs) = self;
        crumbs.pop().map(
            |TreeCrumb {
                 item,
                 mut left,
                 mut right,
             }| {
                left.push(focused);
                left.append(&mut right);
                TreeZipper(Tree::Node(item, left), crumbs)
            },
        )
    }
    /// Get items from the structure located above the current Node.
    pub fn get_ancestors(&self) -> Vec<&T> {
        let TreeZipper(_, crumbs) = self;

        crumbs.iter().map(|c| &c.item).collect()
    }
}

impl<T> TreeZipper<T>
where
    T: Clone + PartialEq,
{
    /// Descend the zipper using `match_fn`, returning a new Zipper at that Node with
    /// crumbs to get back to the source Node if it is satisfied, or None if it isn't.
    pub fn go_to<F>(&self, match_fn: F) -> Option<Self>
    where
        F: Fn(&T) -> bool,
    {
        if let Tree::Node(item, children) = &self.0 {
            for (index, child) in children.iter().enumerate() {
                if let Tree::Node(child_item, _) = child {
                    if match_fn(child_item) {
                        let new_focus = child.clone();
                        let new_crumb = TreeCrumb {
                            item: item.clone(),
                            left: children[0..index].to_vec(),
                            right: children[index + 1..].to_vec(),
                        };
                        let mut new_crumbs = self.1.clone();

                        new_crumbs.push(new_crumb);

                        return Some(TreeZipper(new_focus, new_crumbs));
                    }
                }
            }
        }
        None
    }
}

pub struct BreadthFirstIterator<'a, T>(VecDeque<&'a Tree<T>>);

impl<'a, T> BreadthFirstIterator<'a, T> {
    pub fn new(root: &'a Tree<T>) -> Self {
        let mut queue = VecDeque::new();
        queue.push_back(root);
        Self(queue)
    }
}

impl<'a, T> Iterator for BreadthFirstIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let BreadthFirstIterator(queue) = self;

        queue.pop_front().map(|current| {
            match current {
                Tree::Leaf(value) => {
                    // Leaf nodes have no children to enqueue
                    value
                }
                Tree::Node(value, children) => {
                    // Enqueue all children
                    for child in children {
                        queue.push_back(child);
                    }
                    value
                }
            }
        })
    }
}

pub struct DepthFirstIterator<'a, T>(Vec<&'a Tree<T>>);

impl<'a, T> DepthFirstIterator<'a, T> {
    pub fn new(root: &'a Tree<T>) -> Self {
        let mut stack = Vec::new();
        stack.push(root);
        Self(stack)
    }
}

impl<'a, T> Iterator for DepthFirstIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let Self(stack) = self;

        stack.pop().map(|current| {
            match current {
                Tree::Leaf(value) => {
                    // Leaf nodes have no children to enqueue
                    value
                }
                Tree::Node(value, children) => {
                    // Stack all children
                    for child in children.iter().rev() {
                        stack.push(child);
                    }
                    value
                }
            }
        })
    }
}

pub struct BreadthFirstTreeZipperIterator<T>(VecDeque<TreeZipper<T>>);

// NOTE:
// It's important to communicate that this takes ownership.
// The user should clone themselves. (otherwise this function hides the fact that it is cloning,
// and the user may not want to have the tree cloned).
impl<T: Clone> BreadthFirstTreeZipperIterator<T> {
    pub fn new(root: Tree<T>) -> Self {
        let root_zipper = TreeZipper(root, Vec::new());
        let mut queue = VecDeque::new();
        queue.push_back(root_zipper);

        BreadthFirstTreeZipperIterator(queue)
    }
}

/// PERF:
/// Lots of cloning occurs. Should be investigated and optimized.
impl<T: Clone> Iterator for BreadthFirstTreeZipperIterator<T> {
    type Item = TreeZipper<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let BreadthFirstTreeZipperIterator(queue) = self;

        queue.pop_front().map(|current_zipper| {
            let TreeZipper(focus, crumbs) = &current_zipper;

            match &focus {
                Tree::Leaf(_) => {
                    // No children to add
                }
                Tree::Node(item, children) => {
                    for (i, child) in children.iter().enumerate() {
                        let new_crumb: TreeCrumb<T> = TreeCrumb {
                            item: item.clone(),
                            left: children[0..i].to_vec(),
                            right: children[i + 1..].to_vec(),
                        };
                        let new_zipper =
                            TreeZipper(child.clone(), [crumbs.clone(), vec![new_crumb]].concat());

                        queue.push_back(new_zipper);
                    }
                }
            }
            current_zipper
        })
    }
}

// TODO:
// Make real tests. These just `stdout`
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter_test() {
        // Example usage
        let tree = Tree::Node(
            1,
            vec![
                Tree::Node(2, vec![Tree::Leaf(4), Tree::Leaf(5)]),
                Tree::Node(3, vec![Tree::Leaf(6), Tree::Leaf(7)]),
            ],
        );

        let mut iter = BreadthFirstIterator::new(&tree);

        while let Some(value) = iter.next() {
            println!("{}", value);
        }
    }

    // TODO:
    // Make real tests. These just `stdout`
    #[test]
    fn iter_zipper_test() {
        // Example usage
        let tree = Tree::Node(
            1,
            vec![
                Tree::Node(2, vec![Tree::Leaf(4), Tree::Leaf(5)]),
                Tree::Node(3, vec![Tree::Leaf(6), Tree::Leaf(7)]),
            ],
        );

        let mut iter = BreadthFirstTreeZipperIterator::new(tree);
        while let Some(zipper) = iter.next() {
            // println!("{:#?}", zipper);
        }
    }

    #[test]
    fn depth_iter() {
        let data = Tree::Node(1, vec![
            Tree::Leaf(2),
            Tree::Node(3, vec![
                Tree::Leaf(4),
                Tree::Leaf(5),
            ]),
            Tree::Leaf(6)
        ]);

        assert_eq!(vec![1,2,3,4,5,6], data.iter_depth().map(|&x| x).collect::<Vec<u8>>());
    }
}
