#include <stdio.h>
#include <stdlib.h>

#define FGETS_BUFSIZE 64

/* Problem with FreeBSD 10.3 fgets() with stdin. */
static void
pl_freebsd_fgets()
{
	char buf[FGETS_BUFSIZE];

	if (fgets(buf, FGETS_BUFSIZE, stdin) == NULL)
		exit(1);
}

int
main()
{

	/* Test potential memory leaks. */
	pl_freebsd_fgets();

	/* Success! */
	exit(0);
}
