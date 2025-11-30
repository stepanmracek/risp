(begin
    (define (make-generator start step)
        (begin
            (define val start)
            (lambda ()
                (begin
                    (define result val)
                    (set! val (+ val step))
                    result
                )
            )
        )
    )
    (define next (make-generator 10 -3))
    (display (list (next) (next) (next)))
)
