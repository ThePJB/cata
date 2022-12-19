use std::time::SystemTime;
use std::f32::INFINITY;
use std::f32::consts::SQRT_2;
use crate::priority_queue::*;
use crate::kmath::*;
use crate::kimg::*;
use ordered_float::*;
use std::collections::VecDeque;


// the thing about distance fields is if theres an obstruction it doesnt matter
// it may be doing cheby distance or something
// maybe not so easy for stuff to propagate diagonally
// mayeb second pass needs to do a bigger look

pub fn gen_distance_field_djikstra<F>(f: F, w: usize, h: usize) -> Vec<f32> 
where 
    F: Fn(f32, f32)->bool
{
    let tstart = SystemTime::now();

    let min_spacing = 1.0 / w as f32;
    let sentinel = usize::MAX;
    let mut queue_ops = 0;
    let mut max_queue_len = 0;
    let mut from_table = vec![sentinel; w * h];
    let mut dtable = vec![INFINITY; w * h];
    let mut pq = PriorityQueue::new();
    for i in 0..w {
        for j in 0..h {
            let x = (i as f32 + 0.5) / w as f32;
            let y = (j as f32 + 0.5) / h as f32;
            let walkable = f(x, y);

            // skip open spaces
            if walkable {
                continue;
            }
            let dx = [-1, 0, 1, 0, -1, -1, 1, 1];
            let dy = [0, -1, 0, 1, -1, 1, -1, 1];

            let mut any_open_neighbour = false;
            
            for n in 0..8 {
                let nx = i as i32 + dx[n];
                let ny = j as i32 + dy[n];
                if nx < 0 { continue; }
                if ny < 0 { continue; }
                if nx > w as i32 - 1 { continue; }
                if ny > h as i32 - 1 { continue; }

                let nwalkable = f((nx as f32 + 0.5)/w as f32, (ny as f32 + 0.5)/h as f32);
                if nwalkable {
                    any_open_neighbour = true;
                    break;
                }
            }

            if any_open_neighbour {
                pq.push(OrderedFloat(0.0f32), (i, j));
            }
            dtable[j*w + i] = 0.0;
        }
    }
    while let Some((d, (i, j))) = pq.pop() {
        queue_ops += 1;
        max_queue_len = max_queue_len.max(pq.heap.len());
        let dx = [-1, 0, 1, 0, -1, -1, 1, 1];
        let dy = [0, -1, 0, 1, -1, 1, -1, 1];
        let nd = [min_spacing * 1.0, min_spacing * 1.0, min_spacing * 1.0, min_spacing * 1.0, min_spacing * SQRT_2, min_spacing * SQRT_2, min_spacing * SQRT_2, min_spacing * SQRT_2];
        
        for n in 0..8 {
            let nx = i as i32 + dx[n];
            let ny = j as i32 + dy[n];
            if nx < 0 { continue; }
            if ny < 0 { continue; }
            if nx > w as i32 - 1 { continue; }
            if ny > h as i32 - 1 { continue; }

            let ni = nx as usize;
            let nj = ny as usize;

            let nd = nd[n] + d.0;
            if nd < dtable[nj*w + ni] {
                dtable[nj*w + ni] = nd;
                from_table[nj*w + ni] = j*w + i;
                pq.push(OrderedFloat(nd), (ni, nj));
            }
        }
    }
    let took = SystemTime::now().duration_since(tstart);
    println!("gen di took {:?}, qops: {}, qmax: {}", took.unwrap(), queue_ops, max_queue_len);
    dtable
}

pub fn gen_distance_field_shitty<F>(f: F, w: usize, h: usize) -> Vec<f32> 
where 
    F: Fn(f32, f32)->bool
{
    let tstart = SystemTime::now();

    let mut distances = vec![INFINITY; w * h];

    let mut queue = VecDeque::new();

    for i in 0..w {
        for j in 0..h {
            let x = (i as f32 + 0.5) / w as f32;
            let y = (j as f32 + 0.5) / h as f32;
            let walkable = f(x,y);
            if !walkable {
                queue.push_back((i, j, x, y))
            }
        }
    }
    let mut queue_ops = 0;
    let mut max_queue_len = 0;
    while let Some((i, j, orig_x, orig_y)) = queue.pop_front() {
        queue_ops += 1;
        max_queue_len = max_queue_len.max(queue.len());
        let x = (i as f32 + 0.5) / w as f32;
        let y = (j as f32 + 0.5) / h as f32;

        let dx = x - orig_x;
        let dy = y - orig_y;

        let d = (dx*dx + dy*dy).sqrt();

        if d < distances[j*w + i] {
            distances[j*w + i] = d;
            // push neighbours
            if i > 0 {
                // maybe only if its less than their distance
                if d < distances[j*w + (i-1)] {
                    distances[j*w + i-1] = d + 1.0/w as f32;
                    queue.push_back((i - 1, j, orig_x, orig_y));
                }
            }
            if j > 0 {
                if d < distances[(j-1)*w + (i)] {
                    distances[(j-1)*w + i] = d + 1.0/w as f32;
                    queue.push_back((i, j - 1, orig_x, orig_y));
                }
            }
            if i < w - 1 {
                if d < distances[(j)*w + (i+1)] {
                    distances[j*w + i+1] = d + 1.0/w as f32;
                    queue.push_back((i + 1, j, orig_x, orig_y));
                }
            }
            if j < h - 1 {
                if d < distances[(j+1)*w + (i)] {
                    distances[(j+1)*w + i] = d + 1.0/w as f32;
                    queue.push_back((i, j + 1, orig_x, orig_y));
                }
            }
        }
    }

    let took = SystemTime::now().duration_since(tstart);
    println!("gen shitty took {:?}, qops: {}, qmax: {}", took.unwrap(), queue_ops, max_queue_len);
    distances
}

pub fn gen_distance_field_sep<F>(f: F, w: usize, h: usize) -> Vec<f32> 
where 
    F: Fn(f32, f32)->bool
{
    let tstart = SystemTime::now();

    let mut distances = vec![INFINITY; w * h];
    let mut nearest = vec![(-1, -1); w*h];

    // spread it
    // first set distrances, nearest to self 
    // then iterate and spread left and right
    // in spreading we check our neighbours distance to our nearest and if its better than what they got we update
    // its a subcase of checking distance of every point to every other point

    for j in 0..h {
        for i in 0..w {
            let idx = j*w + i;
            let x = (i as f32 + 0.5) / w as f32;
            let y = (j as f32 + 0.5) / h as f32;
            if !f(x,y) {
                nearest[idx] = (i as i32, j as i32);
                distances[idx] = 0.0;
            }
        }
    }

    // I sort of wish it didnt come to this, but it actually seems to work quite well
    for iter in 0..4 {

        // pgt down
        for j in 0..h-1 {
            let ny = ((j + 1) as f32 + 0.5) / h as f32;
            for i in 0..w {
                let idx = j*w + i;
                if nearest[idx] == (-1, -1) || nearest[idx + w] == nearest[idx] {
                    continue;
                }
                let x = (i as f32 + 0.5) / w as f32;
                let n_d = distances[idx + w];

                let my_nearest_x = (nearest[idx].0 as f32 + 0.5) / w as f32;
                let my_nearest_y = (nearest[idx].1 as f32 + 0.5) / h as f32;

                let dx = my_nearest_x - x;
                let dy = my_nearest_y - ny;
                let d_to_mine = (dx*dx + dy*dy).sqrt();
                if d_to_mine < n_d {
                    distances[idx + w] = d_to_mine;
                    nearest[idx + w] = nearest[idx];
                }
            }
        }

        // pgt up
        for j in (1..h).rev() {
            let ny = ((j - 1) as f32 + 0.5) / h as f32;
            for i in 0..w {
                let idx = j*w + i;
                if nearest[idx] == (-1, -1)  || nearest[idx - w] == nearest[idx] {
                    continue;
                }
                let x = (i as f32 + 0.5) / w as f32;
                let n_d = distances[idx - w];

                let my_nearest_x = (nearest[idx].0 as f32 + 0.5) / w as f32;
                let my_nearest_y = (nearest[idx].1 as f32 + 0.5) / h as f32;

                let dx = my_nearest_x - x;
                let dy = my_nearest_y - ny;
                let d_to_mine = (dx*dx + dy*dy).sqrt();
                if d_to_mine < n_d {
                    distances[idx - w] = d_to_mine;
                    nearest[idx - w] = nearest[idx];
                }
            }
        }


        // pgt right
        for j in 0..h {
            let y = (j as f32 + 0.5) / h as f32;
            for i in 0..w-1 {
                let idx = j*w + i;
                if nearest[idx] == (-1, -1) || nearest[idx + 1] == nearest[idx] {
                    continue;
                }
                let n_d = distances[idx + 1];
                let nx = ((i + 1) as f32 + 0.5) / w as f32;

                let my_nearest_x = (nearest[idx].0 as f32 + 0.5) / w as f32;
                let my_nearest_y = (nearest[idx].1 as f32 + 0.5) / h as f32;

                let dx = my_nearest_x - nx;
                let dy = my_nearest_y - y;
                let d_to_mine = (dx*dx + dy*dy).sqrt();
                if d_to_mine < n_d {
                    distances[idx + 1] = d_to_mine;
                    nearest[idx + 1] = nearest[idx];
                }
            }
        }

        // pgt left
        for j in 0..h {
            let y = (j as f32 + 0.5) / h as f32;
            for i in (1..w).rev() {
                let idx = j*w + i;
                if nearest[idx] == (-1, -1) || nearest[idx - 1] == nearest[idx] {
                    continue;
                }
                let n_d = distances[idx - 1];
                let nx = ((i - 1) as f32 + 0.5) / w as f32;

                let my_nearest_x = (nearest[idx].0 as f32 + 0.5) / w as f32;
                let my_nearest_y = (nearest[idx].1 as f32 + 0.5) / h as f32;

                let dx = my_nearest_x - nx;
                let dy = my_nearest_y - y;
                let d_to_mine = (dx*dx + dy*dy).sqrt();
                if d_to_mine < n_d {
                    distances[idx - 1] = d_to_mine;
                    nearest[idx - 1] = nearest[idx];
                }
            }
        }
    }






    for i in 0..distances.len() {
        if distances[i] == INFINITY {
            distances[i] = 2.0;
            panic!("infinite distance");
        }
    }


    
    let took = SystemTime::now().duration_since(tstart);
    println!("gen sep took {:?}", took.unwrap());
    distances
}

#[test]
fn test_distance_field() {
    let w = 1600;
    let h = 1600;

    let f = |x, y| noise2d(16.0 * x, 16.0 * y, 721) > 0.1;
    let distances = gen_distance_field_djikstra(f, w, h);
    let mut max = 0.0;
    for d in distances.iter() {
        max = max.max(*d);
    }

    let mut imbuf = ImageBuffer::new(w, h);
    for i in 0..w {
        for j in 0..h {
            let val = distances[j*w + i] / max;
            let c = ((255.0 * val) as u8, (255.0 * val) as u8, (255.0 * val) as u8, );
            imbuf.set_px(i, j, c);
        }
    }
    imbuf.dump_to_file("dtest.png");
    let distances = gen_distance_field_shitty(f, w, h);
    let mut max = 0.0;
    for d in distances.iter() {
        max = max.max(*d);
    }

    let mut imbuf = ImageBuffer::new(w, h);
    for i in 0..w {
        for j in 0..h {
            let val = distances[j*w + i] / max;
            let c = ((255.0 * val) as u8, (255.0 * val) as u8, (255.0 * val) as u8, );
            imbuf.set_px(i, j, c);
        }
    }
    imbuf.dump_to_file("dtest2.png");

    let distances = gen_distance_field_sep(f, w, h);
    let mut max = 0.0;
    for d in distances.iter() {
        max = max.max(*d);
    }
    let mut imbuf = ImageBuffer::new(w, h);
    for i in 0..w {
        for j in 0..h {
            let val = distances[j*w + i] / max;
            let c = ((255.0 * val) as u8, (255.0 * val) as u8, (255.0 * val) as u8, );
            imbuf.set_px(i, j, c);
        }
    }
    imbuf.dump_to_file("dtest3.png");
}

#[test]
fn test_dsep() {
    let w = 800;
    let h = 800;

    let f = |x: f32, y: f32| {
        let d = ((x - 0.5)*(x-0.5) + (y-0.5)*(y-0.5)).sqrt();
        if d < 0.1 || d > 0.4 {
            return false;
        }
        return true;
    };

    let distances = gen_distance_field_sep(f, w, h);
    let mut max = 0.0;
    for d in distances.iter() {
        max = max.max(*d);
    }
    let mut imbuf = ImageBuffer::new(w, h);
    for i in 0..w {
        for j in 0..h {
            let val = distances[j*w + i] / max;
            let c = ((255.0 * val) as u8, (255.0 * val) as u8, (255.0 * val) as u8, );
            imbuf.set_px(i, j, c);
        }
    }
    imbuf.dump_to_file("septest.png");    
}