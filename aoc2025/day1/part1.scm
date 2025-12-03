(begin
    (define pos 50)
    (define inputs (split-string (read-file "aoc2025/day1/input")))
    (define inputs (map (lambda (s) (list (substring s 0 1) (string->int (substring s 1 -1)))) inputs))

    (define (dial direction amount)
        (begin
            (if (= direction "R")
                (set! pos (mod (+ pos amount) 100))
                (set! pos (mod (- pos amount) 100))
            )
            pos
        )
    )

    (define positions (map (lambda (input) (apply dial input)) inputs))
    (define zero-count (apply + (map (lambda (input) (if (= input 0) 1 0)) positions)))
    (display zero-count)
)
