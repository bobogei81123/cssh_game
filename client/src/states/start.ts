import Main from '../main';
import * as Phaser from 'phaser-ce';
import Bullet, {Hit} from '../objects/bullet';

const Point = Phaser.Point;
type Point = Phaser.Point;

enum Substate {
    Rest,
    Answer,
    Fire,
    Resolving,
}

export class Start extends Phaser.State {
    substate: Substate;

    constructor(public main: Main) {
        super();
        this.substate = Substate.Rest;
    }

    create() {
        this.registEvents();
    }

    registEvents() {
        this.main.ee.on('Fire', (data) => {
            console.log(data);
            const {fire, damage} = data;
            let bullet;
            if (damage == null) {
                bullet = new Bullet(this.game);
            } else {
                bullet = new Bullet(this.game, new Hit(
                    this.main.data.users[damage.target].position, () => {
                    this.main.data.setHealth(damage.target, damage.health_after);
                }));
            }
            bullet.fire(new Point(fire.pos.x, fire.pos.y), fire.angle);
        });
    }

    setProblemHTML(question, answers) {
        const $modal = document.getElementById('modal');
        const $question = document.getElementById('question');
        const $answers = document.getElementById('answers');

        $question.innerText = question;
        while ($answers.firstChild) $answers.removeChild($answers.firstChild);

        answers.forEach((answer, idx) => {
            const $article = document.createElement("article");
            $article.classList.add('message');
            $article.classList.add('is-primary');

            const $content = document.createElement("div");
            $content.classList.add('message-body');
            $content.innerText = `(${idx+1}) ${answer}`;

            $article.appendChild($content);
            $answers.appendChild($article);
        });
    }

    showProblem() {
        const $modal = document.getElementById('modal');
        $modal.classList.add('is-active');
    }

    hideProblem() {
        const $modal = document.getElementById('modal');
        $modal.classList.remove('is-active');
    }


    update() {
        switch (this.substate) {
            case Substate.Rest: {
                this.substate = Substate.Resolving;
                this.main.send('RequestProblem');
                this.main.ee.once('Problem', ({question, answers}) => {
                    this.substate = Substate.Answer;
                    this.setProblemHTML(question, answers);
                    this.showProblem();
                });
                break;
            }
            case Substate.Answer: {
                break;
            }
            case Substate.Fire: {
                this.substate = Substate.Resolving;
                this.main.startFire().then(() => {
                    this.substate = Substate.Rest;
                });
            }
            case Substate.Resolving: {
                break;
            }
        }
    }
}
