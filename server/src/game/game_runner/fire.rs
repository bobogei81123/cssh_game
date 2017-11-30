use super::*;

#[derive(Serialize, Debug)]
pub struct Damage {
    pub target: Id,
    pub value: f64,
    pub health_after: f64,
    pub dead_after: bool,
}

#[derive(Serialize, Debug)]
pub struct FireResult {
    pub id: Id,
    pub fire: Fire,
    pub target: Option<Id>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Fire {
    pub pos: Point,
    pub angle: f64,
}

fn _get_distant_to_line(o: Point, angle: f64, x: Point) -> (f64, f64) {
    let unit = Point::from_angle(angle);
    let d = x - o;
    let dp = d * unit;
    let dd = f64::sqrt(d * d - dp * dp);

    (dp, dd)
}


impl Game {
    pub fn player_fire(&mut self, id: Id, data: Fire) -> Option<()> {

        let result = {
            let player = self.players.get(&id)?;
            player.alive.as_option()?;

            self.players
                .values()
                .filter(|other| other.alive && other.team != player.team)
                .fold(
                    None,
                    |x, other| {
                        let (dis_par, dis_oth) = _get_distant_to_line(
                            player.pos, data.angle, other.pos
                        );

                        if dis_par < 0. || dis_oth > USER_RADIUS { return x; }

                        match x {
                            None => Some((other.id, dis_par, dis_oth)),
                            Some(best) => {
                                let (_, best_par, _) = best;
                                if best_par > dis_par { Some((other.id, dis_par, dis_oth)) }
                                else { Some(best) }
                            }
                        }
                    }
                )
        };


        let target_id = match result {
            Some((target_id, dis_par, dis_oth)) => {

                let damage = 40. * (USER_RADIUS - dis_oth) / USER_RADIUS + 10.;


                let delta_ms = (FIRE_DELAY + (dis_par / MISSLE_SPEED * 1000.)) as u64;

                self.run_after(box move |me: &mut Self| {
                    let (health_after, dead_after) = {
                        let target = match me.players.get_mut(&target_id) {
                            Some(target) => target,
                            None => { return; }
                        };


                        target.health.sub(damage);
                        (target.health.value, target.health.value <= 0.0)
                    };

                    me.send_many(me.players.keys(), &Output::Damage(Damage {
                        target: target_id,
                        value: damage,
                        health_after: health_after,
                        dead_after: dead_after,
                    }));

                    if (dead_after) {
                        me.user_dead(target_id);
                    }

                }, Duration::from_millis(delta_ms));

                Some(target_id)
            }
            None => None,
        };


        let output = Output::FireResult(FireResult {
            id: id,
            fire: data,
            target: target_id,
        });

        self.send_many(self.players.keys(), &output);

        {
            let player = self.players.get_mut(&id)?;
            player.state = PlayerState::Waiting;
        }
        self.run_after(box move |me: &mut Self| {
            me.assign_problem(id);
        }, Duration::from_secs(1));

        Some(())
    }
}
